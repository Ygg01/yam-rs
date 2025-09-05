use crate::tokenizer::buffers::{YamlBuffer, YamlSource};
use crate::tokenizer::stage2::get_fast_single_quote;
use crate::{branchless_min, EventListener, Stage1Scanner, YamlChunkState, YamlError, YamlResult};
use alloc::vec::Vec;

#[derive(Default)]
pub struct ChunkState {
    // Previous chunk fields
    pub(crate) last_indent: u32,
    pub(crate) last_col: u32,
    pub(crate) last_row: u32,
    pub(crate) previous_indent: u32,
    pub(crate) prev_iter_inside_quote: u64,
    pub(crate) is_indent_running: bool,
    pub(crate) is_previous_white_space: bool,
    pub(crate) is_prev_iter_odd_single_quote: bool,
    pub(crate) is_prev_double_quotes: bool,
    pub(crate) is_in_comment: bool,
    pub(crate) pos: usize,
    pub(crate) prev_char: u8,
    pub(crate) is_not_dummy: bool,
}

/// Represents the internal state of a YAML parser.
///
/// The `YamlParserState` struct is used to track the parser's state as it processes
/// a YAML document. This state includes various counters and flags needed to
/// correctly parse and understand the structure and content of the document.
///
/// # Fields (for internal use only)
///
/// ## State fields:
/// * `state`: current state of the Parser
///
/// ## Structural fields:
/// * `structurals`: A vector of position indices marking structural elements
///   like start and end positions of nodes in the YAML document.
/// * `pos`: The current position in the structural array.
///
/// ## Sparse fields:
/// - `open_close_tag`: A list of all structurals that start or end YAML
/// - `potential_block`: A list of structurals that are potentially valid block tokens.
///
/// ## Previous chunk fields
/// - `last_indent`: The indentation level of the last chunk processed.
/// - `last_col`: The column position of the last chunk processed.
/// - `last_row`: The row position of the last chunk processed.
/// - `previous_indent`: The indentation level before the current chunk.
/// - `Prev_iter_inside_quote`: Tracks the quoting state of the previous iteration
///   to determine the continuation of strings across lines.
/// - `is_indent_running`: A flag indicating if the parser is currently processing
///   an indentation level.
/// - `is_previous_white_space`: Indicates if the last processed character was whitespace.
/// - `is_prev_iter_odd_single_quote`: Tracks if there's an odd number of single quotes
///   up to the previous iteration, affecting string parsing.
/// - `is_prev_double_quotes`: Indicates if the string being parsed is inside double quotes.
/// - `is_in_comment`: A flag that tracks if the parser is currently inside a comment segment.
///
/// This struct is part of the internal workings of a YAML parsing library, often
/// used by the parsing modules such as `stage1` and `stage2` for processing
/// various stages of parsing a YAML document.

#[derive(Default)]
#[allow(clippy::struct_excessive_bools)]
pub struct YamlStructurals {
    /// State field
    pub(crate) state: State,

    /// Structural fields
    pub structurals: Vec<usize>,

    /// Indent of each structural
    pub structural_rows: Vec<usize>,

    /// Position of head in structurals
    pub(crate) pos: usize,

    /// Position of character in the source
    pub(crate) idx: usize,
}

impl YamlStructurals {
    /// Computes the next index in the `structurals` array relative to the current position (`self.pos`)
    /// and ensures it does not exceed the bounds of the `structurals` array.
    ///
    /// # Safety
    /// This function does not perform bound checking and assumes that the `structurals` contain valid position in source array.
    ///
    /// # Examples
    /// ```rust
    /// // Assuming `self` is properly initialized:
    /// let next_value = self.next_idx();
    /// ```
    #[inline]
    #[must_use]
    pub(crate) fn next_struct_idx(&self) -> usize {
        let next_idx = branchless_min!(<usize>, self.pos + 1, self.structurals.len() - 1);

        // SAFETY will always point to the correct position
        unsafe { *self.structurals.get_unchecked(next_idx) }
    }
}

#[derive(Debug, Default)]
pub(crate) enum State {
    #[default]
    PreDocStart,
    AfterDocBlock,
    InDocEnd,
    FlowSeq,
    FlowMap,
    DocBlock,
    BlockSeq,
    BlockMap,
}

impl YamlStructurals {
    pub fn process_chunk<S>(&mut self, chunk_state: &YamlChunkState)
    where
        S: Stage1Scanner,
    {
        // First, we find all interesting structural bits
        S::flatten_bits_yaml(chunk_state, self);
    }

    pub(crate) fn next_state() -> YamlResult<()> {
        todo!()
    }
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

pub(crate) fn run_state_machine<'de, 's: 'de, S, B>(
    parser_state: &mut YamlStructurals,
    event_listener: &mut impl EventListener,
    chunk_state: &mut ChunkState,
    source: &S,
    mut buffer: B,
) -> YamlResult<()>
where
    // E: EventListener<EventListener::Value=&'de [u8]>,
    S: YamlSource<'s>,
    B: YamlBuffer,
{
    let mut idx = 0usize;
    let mut next_idx = 0usize;
    let mut indent = 0;
    let mut chr = b' ';
    let mut i = 0usize;

    macro_rules! update_char {
        () => {
            if parser_state.pos < parser_state.structurals.len() {
                // SAFETY: Safety of `get_unchecked` relies on implementation of Stage1Scanner.
                parser_state.idx = unsafe { *parser_state.structurals.get_unchecked(i) };
                parser_state.pos += 1;
                // SAFETY: Safety of `get_unchecked` relies on implementation of Stage1Scanner.
                chr = unsafe { source.get_u8_unchecked(parser_state.idx) }
            } else {
                break;
            }
        };
    }

    loop {
        update_char!();

        match chr {
            b'"' => {
                // get_fast_double_quote(&source, &mut buffer, indent, event_listener)?;
            }
            b'\'' => {
                get_fast_single_quote(
                    source,
                    &mut buffer,
                    event_listener,
                    chunk_state,
                    parser_state,
                );
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
