use crate::impls::AvxScanner;
use crate::tape::{EventListener, Node};
use crate::tokenizer::stage2::State;
use crate::util::NoopValidator;
use crate::{
    ChunkyIterator, NativeScanner, Stage1Scanner, YamlBuffer, YamlChunkState, YamlError,
    YamlParserState, YamlResult,
};
use alloc::vec::Vec;
use core::str::from_utf8_unchecked;
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

impl<'de> EventListener<'de> for Vec<Node<'de>> {
    type ScalarValue = &'de str;

    fn on_scalar(&mut self, scalar_value: Self::ScalarValue, _scalar_type: ScalarType) {
        self.push(Node::String(scalar_value));
    }
}

trait Source<'s> {
    fn get_span_unsafely(&self, start: usize, end: usize) -> &'s [u8];
}

pub trait Buffer<'b> {
    fn append<'src: 'b>(&mut self, src: &'src [u8]) -> &'b [u8];
}

impl<'b> Buffer<'b> for () {
    fn append<'src: 'b>(&mut self, src: &'src [u8]) -> &'b [u8] {
        src
    }
}

impl<'s> Source<'s> for &'s [u8] {
    fn get_span_unsafely(&self, start: usize, end: usize) -> &'s [u8] {
        unsafe { self.get_unchecked(start..end) }
    }
}

impl<'de> Deserializer<'de> {
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

    pub fn fill_tape(input: &'de str) -> YamlResult<Self> {
        let mut state = YamlParserState::default();
        let mut deserialize = Deserializer {
            idx: 0,
            tape: Vec::new(),
        };

        Self::run_fill_tape_fastest(input, &mut state)?;
        run_state_machine(&mut state, &mut deserialize.tape, input.as_bytes(), ())?;
        Ok(deserialize)
    }

    fn run_fill_tape_fastest(input: &str, mut state: &mut YamlParserState) -> YamlResult<()> {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                // SAFETY: We have detected the feature is enabled at runtime,
                // so it's safe to call this function.
                return Self::fill_tape_inner::<AvxScanner, NoopValidator>(
                    input.as_bytes(),
                    &mut state,
                );
            }
        }
        Self::fill_tape_inner::<NativeScanner, NoopValidator>(input.as_bytes(), &mut state)
    }
}

fn run_state_machine<'de, 's: 'de, E, S, B>(
    parser_state: &mut YamlParserState,
    event_listener: &mut E,
    source: S,
    mut buffer: B,
) -> YamlResult<()>
where
    E: EventListener<'de, ScalarValue = &'de str>,
    S: Source<'s>,
    B: Buffer<'de>,
{
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
                    let x = from_utf8_unchecked(buffer.append(source.get_span_unsafely(0, 3)));
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
fn get_validator<S: Stage1Scanner>() -> impl ChunkedUtf8Validator {
    S::validator()
}

fn get_noop_validator() -> impl ChunkedUtf8Validator {
    NoopValidator()
}
