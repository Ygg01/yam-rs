use crate::tape::{EventListener, MarkedNode, Node};
use crate::tokenizer::buffers::YamlSource;
use crate::util::NoopValidator;
use crate::{
    ChunkyIterator, NativeScanner, Stage1Scanner, YamlBuffer, YamlChunkState, YamlError,
    YamlParserState, YamlResult,
};
use alloc::vec;
use alloc::vec::Vec;
use simdutf8::basic::imp::ChunkedUtf8Validator;
use yam_common::{Mark, ScalarType};

pub(crate) mod buffers;
pub(crate) mod chunk;
pub(crate) mod stage1;
pub(crate) mod stage2;

pub struct Deserializer<'de> {
    idx: usize,
    tape: Vec<Node<'de>>,
}

impl EventListener for Vec<MarkedNode> {
    type Value<'a> = &'a [u8];

    fn on_scalar(&mut self, _: &[u8], scalar_type: ScalarType, mark: Mark) {
        self.push(MarkedNode::String(scalar_type, vec![mark]));
    }

    fn on_scalar_continued(&mut self, _: &[u8], _: ScalarType, mark: Mark) {
        if let Some(MarkedNode::String(_, vec)) = self.last_mut() {
            vec.push(mark);
        }
    }
}

impl<'de> Deserializer<'de> {
    pub fn fill_tape(input: &'de str) -> YamlResult<Self> {
        let mut state = YamlParserState::default();
        let mut mark_tape: Vec<MarkedNode> = Vec::new();

        run_tape_to_end(input, &mut state, &mut mark_tape)?;

        Ok(Self::slice_into_tape(input, mark_tape))
    }

    fn slice_into_tape(_input: &'de str, _vec: Vec<MarkedNode>) -> Deserializer<'de> {
        Deserializer {
            idx: 0,
            tape: vec![],
        }
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
    get_fastest_impl(input, state)?;
    run_state_machine(state, event_listener, input.as_bytes(), ())?;
    Ok(())
}

#[inline]
fn get_fastest_impl(input: &str, state: &mut YamlParserState) -> YamlResult<()> {
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

fn run_state_machine<'de, 's: 'de, E, S, B>(
    parser_state: &mut YamlParserState,
    event_listener: &mut E,
    source: S,
    mut buffer: B,
) -> YamlResult<()>
where
    E: EventListener,
    S: YamlSource<'s>,
    B: YamlBuffer,
{
    let mut idx = 0usize;
    let mut chr = b' ';
    let mut type_of_start = TypeOfDoc::None;
    let mut state;
    let mut i = 0usize;
    macro_rules! update_char {
        () => {
            if i < parser_state.structurals.len() {
                idx = unsafe { *parser_state.structurals.get_unchecked(i) };
                i += 1;
                chr = unsafe { source.get_u8_unchecked(idx) }
            } else {
                return Err(YamlError::Syntax);
            }
        };
    }

    update_char!();
    match chr {
        b'"' => {
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::DoubleQuoted;
        }
        b'-' => {
            // TODO check if its `---` start of YAML
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::Minus;
        }
        b'[' => {
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::FlowArray;
        }
        b'{' => {
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::FlowMap;
        }
        b'?' => {
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::QuestionMark;
        }
        b':' => {
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::Colon;
        }
        b'>' | b'|' => {
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::BlockString {
                is_folded: chr == b'>',
            };
        }
        b'\'' => {
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::SingleQuoted;
        }
        b'.' => {
            type_of_start = TypeOfDoc::Explict;
            event_listener.on_doc_start();
            state = YamlState::OneDot;
        }
        _ => {
            type_of_start = TypeOfDoc::Implict;
            event_listener.on_doc_start();
            state = YamlState::UnQuoted;
        }
    }

    Ok(())
}
