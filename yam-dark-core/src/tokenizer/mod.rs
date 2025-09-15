use crate::tape::{EventListener, MarkedNode, Node};
use crate::tokenizer::buffers::YamlSource;
use crate::tokenizer::parser::run_state_machine;
pub use crate::tokenizer::parser::YamlStructurals;
use crate::tokenizer::stage1::get_fastest_stage1_impl;
use crate::{ChunkState, Stage1Scanner, YamlBuffer, YamlError, YamlResult};
use alloc::borrow::Cow;
use alloc::string::ToString;
use alloc::vec::Vec;
use yam_common::Mark;
pub(crate) mod buffers;
pub(crate) mod chunk;
pub(crate) mod parser;
pub(crate) mod stage1;
pub(crate) mod stage2;

pub struct Deserializer<'de> {
    tape: Vec<Node<'de>>,
}

impl EventListener for Vec<MarkedNode> {
    fn on_scalar(&mut self, _value: &[u8], mark: &Mark) {
        self.push(MarkedNode::StringBorrowed(mark.start..mark.end));
    }

    // fn on_scalar_owned(&mut self, value: Vec<u8>) {
    //     self.push(MarkedNode::StringOwned(value));
    // }
}

impl<'de> Deserializer<'de> {
    pub fn fill_tape(input: &'de str) -> YamlResult<Self> {
        let mut state = YamlStructurals::default();
        let mut mark_tape: Vec<MarkedNode> = Vec::new();

        run_tape_to_end(input, &mut state, &mut mark_tape)?;

        Ok(Self::slice_into_tape(input, mark_tape))
    }

    fn slice_into_tape(input: &'de str, marked_nodes: Vec<MarkedNode>) -> Deserializer<'de> {
        let tape = marked_nodes
            .into_iter()
            .map(|marked_node| match marked_node {
                // MarkedNode::StringBorrowed(Mark { start, end }) => {
                //     // The unsafe relies on YamlParser returning indices that are within scope
                //     Node::String(Cow::Borrowed(unsafe { input.get_unchecked(start..end) }))
                // }
                // MarkedNode::StringOwned(bytes) => {
                //     // The unsafe relies on YamlParser returning indices that are within scope
                //     Node::String(Cow::Owned(unsafe { String::from_utf8_unchecked(bytes) }))
                // }
                MarkedNode::Map { len, count } => Node::Map { len, count },
                MarkedNode::Sequence { len, count } => Node::Sequence { len, count },
                MarkedNode::Static(node) => Node::Static(node),
                _ => Node::String(Cow::Owned("BLA".to_string())),
            })
            .collect();

        Deserializer { tape }
    }
}

/// For a given input string, runs the [`YamlStructurals`] to end, populating the [`EventListener`].
///
/// # Arguments
///
/// * `input`: input strings
/// * `state`: [`YamlStructurals`] that is updated as parser
/// * `event_listener`: event listener to where the events will merge into.
///
/// Returns: [`Result`]<(), `YamlError`> which ends prematurely [`YamlError`] but updates the [`EventListener`] for every successful element reached.
///
/// # Errors
///
/// This function will return an error if there is a YAML parsing error. There are many to list.
#[inline]
pub fn run_tape_to_end<E: EventListener>(
    input: &str,
    state: &mut YamlStructurals,
    event_listener: &mut E,
) -> Result<(), YamlError> {
    get_fastest_stage1_impl(input, state)?;
    let mut chunk = ChunkState::default();
    run_state_machine(state, event_listener, &mut chunk, &input.as_bytes(), ())?;
    Ok(())
}
