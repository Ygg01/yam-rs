use crate::{Stage1Scanner, YamlChunkState, YamlResult};
use alloc::vec::Vec;

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
pub struct YamlParserState {
    /// State field
    pub(crate) state: State,

    /// Structural fields
    pub structurals: Vec<usize>,

    /// Indent of each structural
    pub structural_rows: Vec<usize>,

    /// Position of head in the parser state
    pub pos: usize,

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

impl YamlParserState {
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
