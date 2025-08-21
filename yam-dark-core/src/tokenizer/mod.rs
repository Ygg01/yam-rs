use crate::tape::{EventListener, MarkedNode, Node};
use crate::tokenizer::buffers::YamlSource;
use crate::tokenizer::stage2::Stage2Scanner;
use crate::util::NoopValidator;
use crate::{
    ChunkyIterator, NativeScanner, Stage1Scanner, YamlBuffer, YamlChunkState, YamlError,
    YamlParserState, YamlResult,
};
use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use simdutf8::basic::imp::ChunkedUtf8Validator;
use yam_common::Mark;

pub(crate) mod buffers;
pub(crate) mod chunk;
pub(crate) mod stage1;
pub(crate) mod stage2;

pub struct Deserializer<'de> {
    tape: Vec<Node<'de>>,
}

impl EventListener for Vec<MarkedNode> {
    type Value<'a> = &'a [u8];

    fn on_scalar(&mut self, _value: &[u8], mark: Mark) {
        self.push(MarkedNode::StringBorrowed(mark));
    }

    fn on_scalar_owned(&mut self, value: Vec<u8>) {
        self.push(MarkedNode::StringOwned(value));
    }
}

impl<'de> Deserializer<'de> {
    pub fn fill_tape(input: &'de str) -> YamlResult<Self> {
        let mut state = YamlParserState::default();
        let mut mark_tape: Vec<MarkedNode> = Vec::new();

        run_tape_to_end(input, &mut state, &mut mark_tape)?;

        Ok(Self::slice_into_tape(input, mark_tape))
    }

    fn slice_into_tape(input: &'de str, marked_nodes: Vec<MarkedNode>) -> Deserializer<'de> {
        let tape = marked_nodes
            .into_iter()
            .map(|marked_node| match marked_node {
                MarkedNode::StringBorrowed(Mark { start, end }) => {
                    // The unsafe relies on YamlParser returning indices that are within scope
                    Node::String(Cow::Borrowed(unsafe { input.get_unchecked(start..end) }))
                }
                MarkedNode::StringOwned(bytes) => {
                    // The unsafe relies on YamlParser returning indices that are within scope
                    Node::String(Cow::Owned(unsafe { String::from_utf8_unchecked(bytes) }))
                }
                MarkedNode::Map { len, count } => Node::Map { len, count },
                MarkedNode::Sequence { len, count } => Node::Sequence { len, count },
                MarkedNode::Static(node) => Node::Static(node),
            })
            .collect();

        Deserializer { tape }
    }
}

/// For a given input string, runs the [`YamlParserState`] to end, populating the [`EventListener`].
///
/// # Arguments
///
/// * `input`: input strings
/// * `state`: [`YamlParserState`] that is updated as parser
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
    state: &mut YamlParserState,
    event_listener: &mut E,
) -> Result<(), YamlError> {
    get_fastest_stage1_impl(input, state)?;
    run_state_machine(state, event_listener, input.as_bytes(), ())?;
    Ok(())
}

#[inline]
fn get_fastest_stage1_impl(input: &str, state: &mut YamlParserState) -> YamlResult<()> {
    fn fill_tape_inner<S: Stage1Scanner, V: ChunkedUtf8Validator>(
        input: &[u8],
        state: &mut YamlParserState,
    ) -> YamlResult<()> {
        let mut validator = unsafe { V::new() };
        let mut error_mask = 0;

        for chunk in ChunkyIterator::from_bytes(input) {
            // Invariants:
            // 0. The chunk is always 64 characters long.
            // 1. `validator` is correct for given architecture and parameters
            // 1.1 `validator` can be Noop for &str
            //
            // SAFETY:
            // The `update_from_chunks` function is safe if called on with correct CPU features.
            // It's panic-free if a chunk is a 64-element long array.
            unsafe {
                validator.update_from_chunks(chunk);
            }

            let chunk_state: YamlChunkState = S::next(chunk, state, &mut error_mask);
            state.process_chunk::<S>(&chunk_state);
        }

        if error_mask != 0 {
            return Err(YamlError::Syntax);
        }

        Ok(())
    }

    // #[cfg(target_arch = "x86_64")]
    // {
    //     if is_x86_feature_detected!("avx2") {
    //         // SAFETY: We have detected the feature is enabled at runtime,
    //         // so it's safe to call this function.
    //         return fill_tape_inner::<AvxScanner, NoopValidator>(input.as_bytes(), state);
    //     }
    // }
    fill_tape_inner::<NativeScanner, NoopValidator>(input.as_bytes(), state)
}

#[inline]
fn get_fastest_dq_str<'s, S: YamlSource<'s>, B: YamlBuffer, E: EventListener>(
    source: &S,
    buffer: &mut B,
    indent: i64,
    event_listener: &mut E,
) -> YamlResult<()> {
    fn run_double_quote_inner<
        's,
        A: Stage2Scanner,
        S: YamlSource<'s>,
        B: YamlBuffer,
        E: EventListener,
    >() -> YamlResult<()> {
        //TODO
        Ok(())
    }

    // #[cfg(target_arch = "x86_64")]
    // {
    //     if is_x86_feature_detected!("avx2") {
    //         // SAFETY: We have detected the feature is enabled at runtime,
    //         // so it's safe to call this function.
    //         return fill_tape_inner::<AvxScanner, NoopValidator>(input.as_bytes(), state);
    //     }
    // }
    run_double_quote_inner::<NativeScanner, S, B, E>()
}

enum TypeOfDoc {
    None,
    Implict,
    Explict,
}

impl TypeOfDoc {}

enum YamlState {
    SingleQuoted,
    DoubleQuoted,
    UnQuoted,
    BlockString { is_folded: bool },
    FlowArray,
    FlowMap,
    BlockMap,
    Minus,
    QuestionMark,
    Colon,
    OneDot,
}

fn run_state_machine<'de, 's: 'de, S, B>(
    parser_state: &mut YamlParserState,
    event_listener: &mut impl EventListener,
    source: S,
    mut buffer: B,
) -> YamlResult<()>
where
    // E: EventListener<EventListener::Value=&'de [u8]>,
    S: YamlSource<'s>,
    B: YamlBuffer,
{
    #[unsafe(no_mangle)]
    pub fn unqo(input: &[u8]) -> bool {
        let mut res = false;
        for x in input.split(|x| *x == b'#') {
            res &= x.contains(&b'#');
        }
        res
    }

    let mut idx = 0usize;
    let mut indent = -1;
    let mut chr = b' ';
    let mut i = 0usize;

    macro_rules! update_char {
        () => {
            if i < parser_state.structurals.len() {
                idx = unsafe { *parser_state.structurals.get_unchecked(i) };
                i += 1;
                chr = unsafe { source.get_u8_unchecked(idx) }
            } else {
                break;
            }
        };
    }

    loop {
        update_char!();

        match chr {
            b'"' => {
                get_fastest_dq_str(&source, &mut buffer, indent, event_listener)?;
            }
            b'-' => {
                todo!("Implement start of sequence or start of document")
            }
            b'[' => {
                todo!("Implement start of flow seq")
            }
            b'{' => {
                todo!("Implement start of map states")
            }
            b'?' => {
                todo!("Implement explicit map states")
            }
            b':' => {
                todo!("Implement map states")
            }
            b'>' | b'|' => {
                todo!("Implement block scalars")
            }
            b'\'' => {
                todo!("Implement single quotes")
            }
            b'.' => {
                todo!("Implement dots (DOCUMENT END)")
            }
            _ => {
                todo!("Implement others")
            }
        }
    }

    if !source.has_more() {
        return Err(YamlError::Syntax);
    }

    Ok(())
}
