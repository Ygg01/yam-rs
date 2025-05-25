use crate::tape::{EventListener, Node};
use crate::tokenizer::buffers::BorrowBuffer;
use crate::tokenizer::stage2::State;
use crate::util::NoopValidator;
use crate::{
    ChunkyIterator, NativeScanner, Stage1Scanner, YamlBuffer, YamlChunkState, YamlError,
    YamlParserState, YamlResult,
};
use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::str::from_utf8_unchecked;
use simdutf8::basic::imp::ChunkedUtf8Validator;
use yam_common::ScalarType;

pub(crate) mod buffers;
pub(crate) mod chunk;
pub(crate) mod stage1;
pub(crate) mod stage2;

pub struct Deserializer<'de, B> {
    idx: usize,
    tape: Vec<Node<'de>>,
    source: B,
}

impl<'de> EventListener<'de> for Vec<Node<'de>> {
    type ScalarValue = &'de str;

    fn on_scalar(&mut self, scalar_value: Self::ScalarValue, _scalar_type: ScalarType) {
        self.push(Node::String(scalar_value));
    }
}

fn fill_tape(input: &str) -> YamlResult<()> {
    fn fill_tape_inner<S: Stage1Scanner>(
        input: &[u8],
        state: &mut YamlParserState,
        pre_checked: bool,
    ) -> YamlResult<()> {
        let mut validator = get_validator::<S>(pre_checked);
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
    let mut state = YamlParserState::default();
    fill_tape_inner::<NativeScanner>(input.as_bytes(), &mut state, true)?;

    let mut deserialize = Deserializer {
        idx: 0,
        tape: Vec::new(),
        source: BorrowBuffer::new(input),
    };
    run_state_machine(&mut state, &mut deserialize.tape, &mut deserialize.source)
}

fn run_state_machine<'de, E: EventListener<'de, ScalarValue = &'de str>, B: YamlBuffer<'de>>(
    parser_state: &mut YamlParserState,
    event_listener: &mut E,
    buffer: &'de mut B,
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
                    // let pos = *parser_state.structurals.get_unchecked(parser_state.pos);
                    // buffer.get_byte_unsafely::<usize>(pos)
                    // TODO Remove this
                    let x = from_utf8_unchecked(buffer.get_span_unsafely(0, 3));
                    event_listener.on_scalar(x, ScalarType::Plain);
                    b'3'
                };
                parser_state.pos += 1;
            } else {
                // Return error and defer to clean up.
                break Err(YamlError::UnexpectedEof);
            }
        }
    };

    result
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
