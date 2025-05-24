use crate::impls::AvxScanner;
use crate::tape::{EventListener, Node};
use crate::tokenizer::buffers::BorrowBuffer;
use crate::tokenizer::stage1::NextFn;
use crate::tokenizer::stage2::State;
use crate::util::NoopValidator;
use crate::{
    ChunkyIterator, NativeScanner, Stage1Scanner, YamlBuffer, YamlChunkState, YamlError,
    YamlIndentInfo, YamlParserState, YamlResult,
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use core_detect::is_x86_feature_detected;
use simdutf8::basic::imp::ChunkedUtf8Validator;
use yam_common::ScalarType;

pub(crate) mod buffers;
pub(crate) mod chunk;
pub(crate) mod stage1;
pub(crate) mod stage2;

pub struct Deserializer<'de> {
    idx: usize,
    tape: Vec<Node<'de>>,
}

impl<'de> EventListener<'de> for Deserializer<'de> {
    type ScalarValue = &'de str;

    fn on_scalar(&mut self, scalar_value: Self::ScalarValue, scalar_type: ScalarType) {
        self.tape.push(Node::String(scalar_value));
    }
}

fn fill_tape<'de, T: EventListener<'de>>(input: &'de str, mut deserializer: T) -> YamlResult<()> {
    let mut buffer = BorrowBuffer::new(input);

    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            // SAFETY: We have detected the feature is enabled at runtime,
            // so it's safe to call this function.
            return fill_tape_inner::<AvxScanner, T>(
                input.as_bytes(),
                &mut deserializer,
                &mut buffer,
                true,
            );
        }
    }

    fill_tape_inner::<NativeScanner, T>(input.as_bytes(), &mut deserializer, &mut buffer, true)
}

fn fill_tape_inner<'de, S: Stage1Scanner, E: EventListener<'de>>(
    input: &'de [u8],
    tape: &mut E,
    buffer: &mut BorrowBuffer<'de>,
    pre_checked: bool,
) -> YamlResult<()> {
    let mut iter = ChunkyIterator::from_bytes(input);
    let mut state = YamlParserState::default();
    let mut validator = get_validator::<S>(pre_checked);
    let mut indent_info = YamlIndentInfo::default();

    let next_fn = get_stage1_next::<BorrowBuffer<'de>>();

    for chunk in iter {
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

        let chunk_state: YamlChunkState = S::next(chunk, buffer, &mut state);
        state.process_chunk::<BorrowBuffer<'de>, S>(buffer, &chunk_state, &mut indent_info)?;
    }

    build_tape(&mut state, tape, buffer)
}

fn build_tape<'de, E: EventListener<'de>, B: YamlBuffer<'de>>(
    parser_state: &mut YamlParserState,
    event_listener: &mut E,
    buffer: &mut B,
) -> YamlResult<()> {
    let mut idx = 0;
    let mut chr = b' ';

    let result = loop {
        //early bailout
        if let State::PreDocStart = parser_state.state {
            if parser_state.pos < parser_state.structurals.len() {
                // SAFETY:
                // This method will be safe IFF YamlParserState structurals are safe
                chr = unsafe {
                    let pos = *parser_state.structurals.get_unchecked(parser_state.pos);
                    buffer.get_byte_unsafely::<usize>(pos)
                };
                parser_state.pos += 1;
            } else {
                // Return error and defer to clean up.
                break Err(YamlError::UnexpectedEof);
            }
        }
    };

    // Self::cleanup();

    result
}

#[cfg_attr(not(feature = "no-inline"), inline)]
fn get_stage1_next<'a, B: YamlBuffer<'a>>() -> NextFn<B> {
    NativeScanner::next::<B>
}

/// Function that returns the right validator for the right architecture
///
/// # Arguments
///
/// * `pre_checked`: `true` When working with a [`core::str`] thus not requiring any validation, `false`
///   otherwise. **Note: ** if your [`core::str`] isn't UTF-8 formatted, this will cause Undefined behavior.
///
/// Returns: `Box<dyn ChunkedUtf8Validator + 'static, Global>` a heap allocated [`ChunkedUtf8Validator`] that
/// is guaranteed to be correct for your CPU architecture.
///
/// # Intended use
/// It works on 64-byte arrays, so we use [`ChunkyIterator`] on stable, until
/// [rust#74985](https://github.com/rust-lang/rust/issues/74985) lands.
///
#[cfg_attr(not(feature = "no-inline"), inline)]
fn get_validator<S: Stage1Scanner>(pre_checked: bool) -> Box<dyn ChunkedUtf8Validator> {
    if pre_checked {
        return Box::new(
            // # Invariants:
            //
            // 1. It's correct for currently invoked architecture
            // 2. It will check the bytes for UTF8 validity
            //
            // SAFETY:
            // 1. Doing nothing is safe on every architecture
            // 2. It assumes that bytes are **already** valid UTF8
            unsafe { NoopValidator::new() },
        );
    }

    S::validator()
}
