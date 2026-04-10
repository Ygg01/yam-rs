use crate::prelude::{Marker, Span, Tag, YamlEntry, YamlError};
use crate::{Event, Source, StrSource, YamlDoc, YamlDocAccess, parsing};
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::marker::PhantomData;

/// A struct responsible for loading and parsing YAML documents, while maintaining
/// internal state for tracking document structure and node relationships.
///
/// # Generic Parameters
/// - `'input`: The lifetime of the input data being processed.
/// - `Node`: A type that implements the `LoadableYamlNode` trait and represents
///   a YAML node during the loading process.
///
///
/// # Usage
/// This struct is intended to be used as an entry point for converting input into [`LoadableYamlNode`].
/// Most common yaml node being [`YamlDoc`]
///
/// # Example
/// ```rust
/// use yam_core::prelude::{YamlDoc, YamlDocAccess};
/// use yam_core::YamlLoader;
///
/// let yaml_str = "{a : b, c: d}";
/// let doc = YamlLoader::<YamlDoc>::load_from(yaml_str).expect("Valid input YAML");
///
/// ```
pub struct YamlLoader<'input, Node>
where
    Node: YamlDocAccess<'input>,
{
    docs: Vec<Node>,
    doc_stack: Vec<(Node, usize, Option<Cow<'input, Tag>>)>,
    key_stack: Vec<Node>,
    marker: PhantomData<&'input ()>,
    anchor_map: BTreeMap<usize, Node>,
}

impl<'i, Node> Default for YamlLoader<'i, Node>
where
    Node: YamlDocAccess<'i>,
{
    fn default() -> Self {
        Self {
            docs: Vec::new(),
            doc_stack: Vec::new(),
            key_stack: Vec::new(),
            anchor_map: BTreeMap::new(),
            marker: PhantomData,
        }
    }
}

impl<'input, Node> parsing::SpannedEventReceiver<'input> for YamlLoader<'input, Node>
where
    Node: Clone
        + YamlDocAccess<
            'input,
            Node = Node,
            SequenceNode = Vec<Node>,
            MappingNode = Vec<YamlEntry<'input, Node>>,
        > + From<YamlDoc<'input>>,
{
    fn on_event(&mut self, ev: Event<'input>, span: Span) {
        let marker = span.start;
        match ev {
            Event::DocumentStart(_)
            | Event::Nothing
            | Event::StreamStart
            | Event::StreamEnd
            | Event::Comment(_) => {}
            Event::DocumentEnd => match self.doc_stack.pop() {
                Some((doc, ..)) => self.docs.push(doc),
                None => self.docs.push(Node::bad_span_value(span)),
            },
            Event::SequenceStart(aid, tag) => {
                self.doc_stack.push((
                    Node::from_bare_yaml(YamlDoc::Sequence(Vec::new())).with_start(marker),
                    aid,
                    tag,
                ));
            }
            Event::MappingStart(aid, tag) => {
                self.doc_stack.push((
                    Node::from_bare_yaml(YamlDoc::Mapping(Vec::new())).with_start(marker),
                    aid,
                    tag,
                ));
                self.key_stack.push(Node::bad_span_value(span));
            }
            Event::MappingEnd => {
                self.key_stack.pop();
                self.insert_collection(marker);
            }
            Event::SequenceEnd => {
                self.insert_collection(marker);
            }
            Event::Scalar(parsing::ScalarValue {
                value,
                anchor_id,
                tag,
                scalar_type,
            }) => {
                let node =
                    Node::from_bare_yaml(YamlDoc::from_cow_and_tag(value, scalar_type, &tag));
                self.insert_new_node(node, anchor_id, tag);
            }
            Event::Alias(anchor_id) => {
                let node = match self.anchor_map.get(&anchor_id) {
                    Some(n) => n.clone(),
                    None => Node::bad_span_value(span),
                };
                self.insert_new_node(node, anchor_id, None);
            }
        }
    }
}

impl<'input, Node> YamlLoader<'input, Node>
where
    Node: YamlDocAccess<
            'input,
            Node = Node,
            SequenceNode = Vec<Node>,
            MappingNode = Vec<YamlEntry<'input, Node>>,
        > + for<'a> From<YamlDoc<'input>>,
{
    #[must_use]
    pub fn into_documents(self) -> Vec<Node> {
        self.docs
    }

    pub(crate) fn insert_new_node(
        &mut self,
        mut node: Node,
        anchor_id: usize,
        tag: Option<Cow<'input, Tag>>,
    ) {
        if anchor_id > 0 {
            self.anchor_map.insert(anchor_id, node.clone());
        }
        if let Some((parent_node, _, _)) = self.doc_stack.last_mut() {
            if let Some(tag) = tag
                && node.is_collection()
                && !tag.is_yaml_core_schema()
            {
                node = node.into_tagged(tag);
            }
            if parent_node.is_sequence() {
                parent_node.sequence_mut().push(node);
            } else if parent_node.is_mapping() {
                let curr_key = self.key_stack.last_mut().unwrap();

                if curr_key.is_bad_value() {
                    *curr_key = node;
                } else {
                    parent_node
                        .mapping_mut()
                        .push(YamlEntry::new(curr_key.take(), node));
                }
            }
        } else {
            self.doc_stack.push((node, anchor_id, tag));
        }
    }

    fn insert_collection(&mut self, marker: Marker) {
        if let Some((mut node, anchor_id, tag)) = self.doc_stack.pop() {
            node = node.with_end(marker);
            if let Some(tag) = tag
                && !tag.is_yaml_core_schema()
            {
                node = node.into_tagged(tag);
            }
            self.insert_new_node(node, anchor_id, None);
        }
    }

    ///
    /// Loads a sequence of YAML documents from a parser instance and returns them as a vector of `Node`s.
    ///
    /// # Type Parameters
    /// - `I`: A type that implements the `Source` trait, representing the source input for the parser.
    ///
    /// # Arguments
    /// - `parser`: A mutable reference to a `Parser` instance that processes the YAML input.
    ///
    /// # Returns
    /// - `Ok(Vec<Node>)`: A vector of `Node` objects representing the parsed YAML documents.
    /// - `Err(YamlError)`: An error if the parsing process fails.
    ///
    /// # Errors
    /// This function will return a `YamlError` if the parser encounters issues while parsing and processing input.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::{YamlLoader, YamlDoc};
    /// use yam_core::parsing::{Parser};
    ///
    /// let mut parser = Parser::new_from_str("a: b");
    ///
    /// match YamlLoader::<YamlDoc>::load_from_parser(&mut parser) {
    ///     Ok(documents) => {
    ///         for node in documents {
    ///             println!("{:?}", node);
    ///         }
    ///     }
    ///     Err(e) => {
    ///         eprintln!("Failed to parse YAML: {}", e);
    ///     }
    /// }
    /// ```
    pub fn load_from_parser<I: Source>(
        parser: &mut parsing::Parser<'input, I>,
    ) -> Result<Vec<Node>, YamlError> {
        let mut loader = YamlLoader::default();
        parser.load(&mut loader, true)?;
        Ok(loader.into_documents())
    }

    ///
    /// Parses the given YAML input string and loads it into a `Vec<Node>`.
    ///
    /// # Type Parameters
    /// - `S`: A type that can be referenced as a string slice, implementing the `AsRef<str>` trait.
    ///
    /// # Arguments
    /// - `input`: A YAML input source, provided as a type that can be referenced as a string.
    ///
    /// # Returns
    /// - `Ok(Vec<Node>)`: A vector of `Node` objects representing the YAML documents parsed from the input.
    /// - `Err(YamlError)`: An error if the parsing process fails.
    ///
    /// # Errors
    /// Returns a `YamlError` if:
    /// - The input contains invalid YAML syntax.
    /// - Parsing fails due to other reasons, such as malformed structures in the YAML document.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::parsing::{Parser};
    /// use yam_core::prelude::{YamlDoc, YamlLoader, YamlDocAccess};
    ///
    /// let yaml_input = r#"
    /// - name: John
    ///   age: 30
    /// - name: Jane
    ///   age: 25
    /// "#;
    ///
    /// let result = YamlLoader::<YamlDoc>::load_from(&yaml_input);
    /// match result {
    ///     Ok(nodes) => {
    ///         for node in nodes {
    ///             println!("{:?}", node);
    ///         }
    ///     }
    ///     Err(err) => eprintln!("Failed to parse YAML: {}", err),
    /// }
    /// ```
    pub fn load_from<S: AsRef<str>>(input: S) -> Result<Vec<Node>, YamlError> {
        let mut event_listener = YamlLoader::default();
        let mut parser = parsing::Parser::new(StrSource::new(input.as_ref()));
        parser.load(&mut event_listener, true)?;
        Ok(event_listener.docs)
    }

    ///
    /// Loads a single YAML document from the given input string.
    ///
    /// # Type Parameters
    /// - `S`: A type that can be converted into a string slice (`&str`)
    ///   using the `AsRef<str>` trait, typically `String` or `&str`.
    ///
    /// # Parameters
    /// - `input`: The YAML input as a string-like object. This is the source from
    ///   which the function attempts to parse a single YAML document.
    ///
    /// # Returns
    /// - `Ok(Node)`: On success, returns the first YAML document (`Node`) parsed from the input string.
    /// - `Err(YamlError)`: Returns an error if:
    ///   - The input is invalid or cannot be parsed due to syntax or structural issues.
    ///   - No YAML document is found within the input (`YamlError::NoDocument`).
    ///
    /// # Errors
    /// - `YamlError::NoDocument`: Returned if no valid YAML documents are found in the input.
    /// - Any other error encountered during parsing, as propagated from the parser.
    ///
    /// # Example
    /// ```
    /// use yam_core::parsing::Parser;
    /// use yam_core::{YamlLoader, YamlDoc};
    ///
    /// let yaml_input = r#"
    /// key: value
    /// array:
    ///   - item1
    ///   - item2
    /// "#;
    ///
    /// match YamlLoader::<YamlDoc>::load_single(&yaml_input) {
    ///     Ok(node) => println!("Parsed Node: {:?}", node),
    ///     Err(e) => eprintln!("Error loading YAML: {}", e),
    /// }
    /// ```
    ///
    /// # Notes
    /// This function expects only a single YAML document in the input. Any additional
    /// documents beyond the first will be ignored.
    ///
    pub fn load_single<S: AsRef<str>>(input: S) -> Result<Node, YamlError> {
        let mut event_listener = YamlLoader::default();
        let mut parser = parsing::Parser::new(StrSource::new(input.as_ref()));
        parser.load(&mut event_listener, false)?;
        event_listener
            .docs
            .first()
            .cloned()
            .ok_or(YamlError::NoDocument)
    }

    ///
    /// Parses a single YAML document from the given input source and returns it as a `Node`.
    ///
    /// # Type Parameters
    /// - `I`: A type that implements the `Source` trait, representing the input data source.
    ///
    /// # Arguments
    /// - `parser`: The input source containing the [`crate::parsing::Parser`] that parses the input.
    ///
    /// # Returns
    /// - `Ok(Node)`: The parsed YAML document represented as a `Node` if successful.
    /// - `Err(YamlError)`: An error if no document is found in the input or if parsing fails.
    ///
    /// # Errors
    /// - `YamlError::NoDocument`: Returned if there is no document available in the parsed input.
    /// - Other variants of `YamlError`: Returned if there is an issue during parsing.
    ///
    /// # Example
    /// ```
    /// use yam_core::parsing::Parser;
    /// use yam_core::prelude::*;
    ///
    /// let yaml_input = r#"
    /// key: value
    /// array:
    ///   - item1
    ///   - item2
    /// "#;
    /// let mut parser = Parser::new_from_str(&yaml_input);
    /// match YamlLoader::<YamlDoc>::load_single_from_parser(&mut parser) {
    ///     Ok(node) => println!("Parsed Node: {:?}", node),
    ///     Err(e) => eprintln!("Error loading YAML: {}", e),
    /// }
    /// ```
    /// # Notes
    /// This function expects only a single YAML document in the input. Any additional
    /// documents beyond the first will be ignored.
    ///
    pub fn load_single_from_parser<I: Source>(
        parser: &mut parsing::Parser<'input, I>,
    ) -> Result<Node, YamlError> {
        let mut event_listener = YamlLoader::default();
        parser.load(&mut event_listener, false)?;
        event_listener
            .docs
            .first()
            .cloned()
            .ok_or(YamlError::NoDocument)
    }

    ///
    /// Loads a single YAML document from the given input source.
    ///
    /// This function takes an input source that implements the `Source` trait
    /// and attempts to parse it into a single YAML document. If parsing succeeds,
    /// it returns a `Node` representing the document. If no document is found or
    /// the parsing fails, an appropriate error is returned.
    ///
    /// # Type Parameters
    /// - `I`: A type that implements the `Source` trait, representing the input source.
    ///
    /// # Arguments
    /// - `input`: An input source that provides the YAML content to be loaded.
    ///
    /// # Returns
    /// - `Ok(Node)`: The first parsed YAML document as a `Node`.
    /// - `Err(YamlError)`: An error if no document is found or parsing fails. Possible errors include:
    ///   - `YamlError::NoDocument`: No valid YAML document was found in the input.
    ///   - Other `YamlError` variants describing parsing issues.
    ///
    /// # Errors
    /// This function will return an error in any of the following cases:
    /// - The input source does not contain a valid YAML document.
    /// - Parsing fails due to syntactical issues in the YAML content.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::{YamlDoc, YamlLoader, StrSource};
    ///
    /// let input = StrSource::new("---\nkey: value\n");
    /// match YamlLoader::<YamlDoc>::load_single_source(input) {
    ///     Ok(node) => println!("Successfully loaded: {:?}", node),
    ///     Err(e) => eprintln!("Failed to load YAML: {:?}", e),
    /// }
    /// ```
    ///
    /// # Note
    /// The function only extracts the first document if the input source contains multiple YAML documents.
    /// If you need to handle multiple documents, consider using appropriate methods.
    ///
    pub fn load_single_source<I: Source>(input: I) -> Result<Node, YamlError> {
        let mut event_listener = YamlLoader::default();
        let mut parser = parsing::Parser::new(input);
        parser.load(&mut event_listener, false)?;
        event_listener
            .docs
            .first()
            .cloned()
            .ok_or(YamlError::NoDocument)
    }
}
