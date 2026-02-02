pub mod node;
pub mod tree;

use crate::Span;
use crate::saphyr_tokenizer::Event;
use crate::saphyr_tokenizer::{ScalarValue, SpannedEventReceiver};
use crate::treebuild::node::LoadableYamlNode;
use alloc::borrow::Cow;
use alloc::collections::BTreeMap;
use alloc::vec::Vec;
use core::marker::PhantomData;
use yam_common::{Marker, Tag, YamlDoc};

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
                ..
            }) => {
                let node =
                    Node::from_bare_yaml(YamlDoc::from_cow_and_tag(value, &tag)).with_start(marker);
                self.insert_new_node(node, anchor_id, tag)
            }
            Event::Alias(anchor_id) => {
                let n = match self.anchor_map.get(&anchor_id) {
                    Some(n) => n.clone(),
                    None => Node::bad(span),
                };
                self.insert_new_node(n, anchor_id, None)
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

    pub(crate) fn insert_new_node(&self, _node: Node, _anchor_id: usize, _tag: Option<Cow<Tag>>) {
        todo!()
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

    // pub fn load_from<'a, S: AsRef<str>>(input: S) -> Result<Vec<YamlDoc<'a>>, YamlError> {
    //     let mut event_listener = YamlLoader::<Node>::default();
    //     let mut parser = Parser::new(StrSource::new(input.as_ref()));
    //     parser.load(&mut event_listener, true)?;
    //     Ok(event_listener.docs)
    // }
    //
    // pub fn load_single<'a, S: AsRef<str>>(input: S) -> Result<YamlDoc<'a>, YamlError> {
    //     let mut event_listener = YamlLoader::default();
    //     let mut parser = Parser::new(StrSource::new(input.as_ref()));
    //     parser.load(&mut event_listener, false)?;
    //     event_listener.docs.first().cloned().ok_or(YamlError::NoDocument)
    // }
}
