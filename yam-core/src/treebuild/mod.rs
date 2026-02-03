pub mod spanned_node;

use crate::Parser;
use crate::saphyr_tokenizer::{Event, Source, StrSource};
use crate::saphyr_tokenizer::{ScalarValue, SpannedEventReceiver};
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::marker::PhantomData;
use yam_common::loader::LoadableYamlNode;
use yam_common::{Marker, Span, Tag, YamlDoc, YamlEntry, YamlError};

pub struct YamlLoader<'input, Node>
where
    Node: LoadableYamlNode<'input>,
{
    docs: Vec<Node>,
    doc_stack: Vec<(Node, usize, Option<Cow<'input, Tag>>)>,
    key_stack: Vec<Node>,
    marker: PhantomData<&'input ()>,
    anchor_map: BTreeMap<usize, Node>,
}

impl<'i, Node> Default for YamlLoader<'i, Node>
where
    Node: LoadableYamlNode<'i>,
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

impl<'input, Node> SpannedEventReceiver<'input> for YamlLoader<'input, Node>
where
    Node: LoadableYamlNode<'input>,
{
    fn on_event(&mut self, ev: Event<'input>, span: Span) {
        let marker = span.start;
        match ev {
            Event::DocumentStart(_) | Event::Nothing | Event::StreamStart | Event::StreamEnd => {}
            Event::DocumentEnd => match self.doc_stack.pop() {
                Some((doc, ..)) => self.docs.push(doc),
                None => self.docs.push(Node::bad(span)),
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
                self.key_stack.push(Node::bad(span))
            }
            Event::MappingEnd => {
                self.key_stack.pop();
                self.insert_collection(marker);
            }
            Event::SequenceEnd => {
                self.insert_collection(marker);
            }
            Event::Scalar(ScalarValue {
                value,
                anchor_id,
                tag,
                scalar_type,
            }) => {
                let node =
                    Node::from_bare_yaml(YamlDoc::from_cow_and_tag(value, scalar_type, &tag));
                self.insert_new_node(node, anchor_id, tag)
            }
            Event::Alias(anchor_id) => {
                let node = match self.anchor_map.get(&anchor_id) {
                    Some(n) => n.clone(),
                    None => Node::bad(span),
                };
                self.insert_new_node(node, anchor_id, None)
            }
        };
    }
}

impl<'input, Node> YamlLoader<'input, Node>
where
    Node: LoadableYamlNode<'input>,
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
            self.insert_new_node(node, anchor_id, None)
        }
    }

    pub fn load_from_parser<I: Source>(
        parser: &mut Parser<'input, I>,
    ) -> Result<Vec<Node>, YamlError> {
        let mut loader = YamlLoader::default();
        parser.load(&mut loader, true)?;
        Ok(loader.into_documents())
    }

    pub fn load_from<S: AsRef<str>>(input: S) -> Result<Vec<Node>, YamlError> {
        let mut event_listener = YamlLoader::default();
        let mut parser = Parser::new(StrSource::new(input.as_ref()));
        parser.load(&mut event_listener, true)?;
        Ok(event_listener.docs)
    }

    pub fn load_single<S: AsRef<str>>(input: S) -> Result<Node, YamlError> {
        let mut event_listener = YamlLoader::default();
        let mut parser = Parser::new(StrSource::new(input.as_ref()));
        parser.load(&mut event_listener, false)?;
        event_listener
            .docs
            .first()
            .cloned()
            .ok_or(YamlError::NoDocument)
    }
}
