use crate::node::yaml_data::YamlData;
use crate::parsing;
use crate::parsing::Tag;
use crate::parsing::{Event, ScalarValue, SpannedEventReceiver};
use crate::prelude::{
    IsEmpty, Marker, Source, Span, StrSource, YamlDocAccess, YamlEntry, YamlError, YamlScalar,
};
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;

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
/// This struct is intended to be used as an entry point for converting input into
/// a [`YamlDocAccess`] like node.
///
/// # Example
/// ```rust
/// use yam_core::prelude::{Yaml, YamlLoader, YamlDocAccess};
///
/// let yaml_str = "{a : b, c: d}";
/// let doc = YamlLoader::<Yaml>::load_from(yaml_str);
///
/// ```
pub struct YamlLoader<'input, Node>
where
    Node: YamlDocAccess<'input>,
{
    docs: Vec<Node>,
    doc_stack: Vec<(Node, usize, Option<Cow<'input, Tag>>)>,
    key_stack: Vec<Node>,
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
        }
    }
}

pub trait SequenceLike<T>: IsEmpty {
    fn new_empty() -> Self;

    fn push_elem(&mut self, elem: T);

    fn vec(&self) -> &Vec<T>;
}

impl<T> SequenceLike<T> for Vec<T>
where
    T: Clone,
{
    fn new_empty() -> Self {
        Vec::new()
    }

    fn push_elem(&mut self, elem: T) {
        self.push(elem);
    }

    fn vec(&self) -> &Vec<T> {
        self
    }
}

pub trait MappingLike<T> {
    fn new_map() -> Self;

    fn push_mapping(&mut self, key: T, value: T);

    fn entries(&self) -> &Vec<YamlEntry<'_, T>>;
}

impl<'a, T> MappingLike<T> for Vec<YamlEntry<'a, T>> {
    fn new_map() -> Self {
        Vec::new()
    }

    fn push_mapping(&mut self, key: T, value: T) {
        self.push(YamlEntry {
            key,
            value,
            _marker: Default::default(),
        })
    }

    fn entries(&self) -> &Vec<YamlEntry<'_, T>> {
        self
    }
}

impl<'input, Node, SEQ, MAP> YamlLoader<'input, Node>
where
    Node: YamlDocAccess<'input, OutNode = Node, SequenceNode = SEQ, MappingNode = MAP>
        + From<YamlData<'input, Node>>
        + From<YamlScalar<'input>>,
    SEQ: SequenceLike<Node> + IsEmpty + Clone,
    MAP: MappingLike<Node> + IsEmpty + Clone,
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
                parent_node.sequence_mut().push_elem(node);
            } else if parent_node.is_mapping() {
                let curr_key = self.key_stack.last_mut().unwrap();

                if curr_key.is_bad_value() {
                    *curr_key = node;
                } else {
                    parent_node
                        .mapping_mut()
                        .push_mapping(curr_key.take(), node);
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
    /// # parser`: A mutable reference to a `Parser` instance that processes the YAML input.
    //     ///
    //     /// # RetArguments
    /// - `urns
    /// - `Ok(Vec<Node>)`: A vector of `Node` objects representing the parsed YAML documents.
    /// - `Err(YamlError)`: An error if the parsing process fails.
    ///
    /// # Errors
    /// This function will return a `YamlError` if the parser encounters issues while parsing and processing input.
    ///
    /// # Example
    /// ```rust
    /// use yam_core::prelude::{Yaml, YamlLoader};
    /// use yam_core::parsing::{Parser};
    ///
    /// let mut parser = Parser::new_from_str("a: b");
    ///
    /// match YamlLoader::<Yaml>::load_from_parser(&mut parser) {
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
    /// use yam_core::prelude::{Yaml, YamlLoader, YamlDocAccess};
    ///
    /// let yaml_input = r#"
    /// - name: John
    ///   age: 30
    /// - name: Jane
    ///   age: 25
    /// "#;
    ///
    /// let result = Yaml::load_from(&yaml_input);
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
    /// use yam_core::prelude::{Yaml, YamlLoader};
    ///
    /// let yaml_input = r#"
    /// key: value
    /// array:
    ///   - item1
    ///   - item2
    /// "#;
    ///
    /// match YamlLoader::<Yaml>::load_single(&yaml_input) {
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
    /// - `parser`: The input source containing the [`parsing::Parser`] that parses the input.
    ///
    /// # Returns
    /// - `Ok(Node)`: The parsed YAML document is represented as a `Node` if successful.
    /// - `Err(YamlError)`: An error if it is not found in the input or if parsing fails.
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
    /// match YamlLoader::<Yaml>::load_single_from_parser(&mut parser) {
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
    /// use yam_core::prelude::{YamlLoader, Yaml, StrSource};
    ///
    /// let input = StrSource::new("---\nkey: value\n");
    /// match YamlLoader::<Yaml>::load_single_source(input) {
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

impl<'input, Node, SEQ, MAP> SpannedEventReceiver<'input> for YamlLoader<'input, Node>
where
    Node: YamlDocAccess<'input, OutNode = Node, MappingNode = MAP, SequenceNode = SEQ>
        + From<YamlData<'input, Node>>
        + From<YamlScalar<'input>>,
    SEQ: SequenceLike<Node> + Clone + IsEmpty,
    MAP: MappingLike<Node> + Clone + IsEmpty,
{
    fn on_event(&mut self, ev: Event<'input>, span: Span) {
        let mark = span.start;
        match ev {
            Event::DocumentStart(_) | Event::Nothing | Event::StreamStart | Event::StreamEnd => {
                // do nothing
            }
            Event::DocumentEnd => {
                match self.doc_stack.len() {
                    // empty document
                    0 => self.docs.push(YamlData::BadValue.into()),
                    1 => self.docs.push(self.doc_stack.pop().unwrap().0),
                    _ => unreachable!(),
                }
            }
            Event::SequenceStart(aid, tag) => {
                let node: Node = YamlData::Sequence(Vec::new()).into();

                self.doc_stack.push((node.with_start(mark), aid, tag));
            }
            Event::MappingStart(aid, tag) => {
                let node: Node = YamlData::Mapping(Vec::new()).into();
                self.doc_stack.push((node.with_start(mark), aid, tag));
                self.key_stack.push(YamlData::BadValue.into());
            }
            Event::MappingEnd | Event::SequenceEnd => {
                if ev == Event::MappingEnd {
                    self.key_stack.pop().unwrap();
                }

                let (mut node, anchor_id, tag) = self.doc_stack.pop().unwrap();
                node = node.with_end(mark);
                if let Some(tag) = tag
                    && !tag.is_yaml_core_schema()
                {
                    node = node.into_tagged(tag);
                }
                self.insert_new_node(node, anchor_id, None);
            }
            Event::Scalar(ScalarValue {
                value,
                scalar_type,
                anchor_id,
                tag,
            }) => {
                let node =
                    YamlData::value_from_cow_and_metadata(value, scalar_type, tag.as_ref()).into();
                self.insert_new_node(node, anchor_id, tag);
            }
            Event::Alias(id) => {
                let n = match self.anchor_map.get(&id) {
                    Some(v) => v.clone(),
                    None => YamlData::BadValue.into(),
                };
                self.insert_new_node(n.with_span(span), 0, None);
            }
            Event::Comment(_) => {}
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Yaml, YamlLoader};
    use alloc::borrow::ToOwned;
    use alloc::vec::Vec;

    #[test]
    fn test_simple() {
        let yaml_str = "{a : b, c: d}".to_owned();
        let doc: Vec<Yaml> = YamlLoader::load_from(yaml_str).unwrap();
    }
}
