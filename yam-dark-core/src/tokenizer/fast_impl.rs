use crate::tokenizer::buffers::{YamlBuffer, YamlSource};
use crate::tokenizer::stage2::Stage2Scanner;
use crate::util::NoopValidator;
use crate::{
    ChunkyIterWrap, EventListener, NativeScanner, Stage1Scanner, YamlChunkState, YamlError,
    YamlParserState, YamlResult,
};
use simdutf8::basic::imp::ChunkedUtf8Validator;

#[inline]
pub(crate) fn get_fastest_stage1_impl(input: &str, state: &mut YamlParserState) -> YamlResult<()> {
    fn fill_tape_inner<S: Stage1Scanner, V: ChunkedUtf8Validator>(
        input: &[u8],
        state: &mut YamlParserState,
    ) -> YamlResult<()> {
        let mut validator = unsafe { V::new() };
        let mut error_mask = 0;
        let mut iter = ChunkyIterWrap::from_bytes(input);

        for chunk in iter.by_ref() {
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
        // let chunk = iter.remaining_chunk();
        // let chunk_state = S::next(&chunk, state, &mut error_mask);
        // state.process_chunk::<S>(&chunk_state);

        if error_mask != 0 {
            return Err(YamlError::Syntax);
        }

        Ok(())
    }

    // TODO enable more implementations
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
pub(crate) fn get_fast_double_quote<'s, S: YamlSource<'s>, B: YamlBuffer, E: EventListener>(
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

#[inline]
pub(crate) fn get_fast_single_quote<'s, S: YamlSource<'s>, B: YamlBuffer, E: EventListener>(
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

#[inline]
pub(crate) fn get_fast_block_scalar<'s, S: YamlSource<'s>, B: YamlBuffer, E: EventListener>(
    source: &S,
    buffer: &mut B,
    indent: i64,
    event_listener: &mut E,
) -> YamlResult<()> {
    Ok(())
}

#[inline]
pub(crate) fn get_fast_unquoted_scalar<'s, S: YamlSource<'s>, B: YamlBuffer, E: EventListener>(
    source: &S,
    buffer: &mut B,
    indent: i64,
    event_listener: &mut E,
) -> YamlResult<()> {
    Ok(())
}
