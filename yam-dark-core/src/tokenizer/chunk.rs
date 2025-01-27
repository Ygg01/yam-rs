#[allow(unused_imports)] // imports are used in tests
use crate::{u8x64_eq, NativeScanner, Stage1Scanner, YamlParserState};
use alloc::vec;
use alloc::vec::Vec;

/// Represents the state of YAML chunk processing.
///
/// `YamlChunkState` is used to track the state of various chunks of YAML content,
/// including double-quoted strings, single-quoted strings, and character classifications
/// such as whitespace and structural characters.
///
/// This struct also maintains vectors for row and column positions and
/// indent levels, which are updated as the YAML content is processed.
///
/// # Fields
///
/// * `double_quote` - [`YamlDoubleQuoteChunk`] struct containing double-quoted YAML strings information.
/// * `single_quote` - [`YamlSingleQuoteChunk`] struct containing single-quoted YAML strings information.
/// * `characters` - [`YamlCharacterChunk`] struct containing info for characters (e.g., whitespace, operators).
/// * `rows` - [`Vec`] maintaining the row positions in the chunk.
/// * `cols` - [`Vec`] maintaining the column positions in the chunk.
/// * `indents` - [`Vec`] maintaining the indent levels in the chunk.
/// * `follows_non_quote_scalar` - Bitmask indicating positions following non-quote scalar values.
/// * `error_mask` - Bitmask indicating positions with errors.
///
/// # Default Implementation
///
/// The `Default` implementation for `YamlChunkState` initializes
/// the vectors (`rows`, `cols`, `indents`) to be 64 elements long
/// and sets the `double_quote`, `single_quote`, `characters` to its default values.
///
/// ```rust
/// use yam_dark_core::YamlChunkState;
/// let x = YamlChunkState::default();
/// ```
pub struct YamlChunkState {
    pub double_quote: YamlDoubleQuoteChunk,
    pub single_quote: YamlSingleQuoteChunk,
    pub characters: YamlCharacterChunk,
    pub rows: Vec<u32>,
    pub cols: Vec<u32>,
    pub indents: Vec<usize>,
    pub(crate) error_mask: u64,
}

impl Default for YamlChunkState {
    fn default() -> Self {
        // Safety
        // To ensure cols/rows/indents are always 64 elements long.
        YamlChunkState {
            double_quote: YamlDoubleQuoteChunk::default(),
            single_quote: YamlSingleQuoteChunk::default(),
            characters: YamlCharacterChunk::default(),
            rows: vec![0; 64],
            cols: vec![0; 64],
            indents: vec![0; 64],
            error_mask: 0,
        }
    }
}

#[derive(Default)]
/// Represents the state of double-quoted YAML string processing.
///
/// `YamlDoubleQuoteChunk` is used to track the state of double-quoted YAML strings,
/// maintaining information about escaped characters, real double quotes, and whether
/// characters are within the string context.
///
/// # Fields
///
/// * `escaped` - A bitmask indicating the positions of escaped characters.
/// * `quote_bits` - A bitmask indicating the positions of real double quotes.
/// * `in_string` - A bitmask showing which characters are currently within a double-quoted string.
///
/// # Default Implementation
///
/// The `Default` implementation for `YamlDoubleQuoteChunk` initializes
/// the fields `escaped`, `quote_bits`, and `in_string` to 0.
///
/// ```rust
/// use yam_dark_core::YamlDoubleQuoteChunk;
/// let y = YamlDoubleQuoteChunk::default();
/// assert_eq!(y.escaped, 0);
/// assert_eq!(y.quote_bits, 0);
/// assert_eq!(y.in_string, 0);
/// ```
pub struct YamlDoubleQuoteChunk {
    /// Escaped characters
    pub escaped: u64,
    /// Real double quotes
    pub quote_bits: u64,
    /// Bitmask showing which characters are in string
    pub in_string: u64,
}

#[derive(Default)]
///
/// Represents the state of single-quoted YAML string processing.
///
/// `YamlSingleQuoteChunk` is used to track the state of single-quoted YAML strings,
/// maintaining information about odd quotes, escaped quotes, and whether
/// characters are within the string context.
///
/// # Fields
///
/// * `odd_quotes` - A bitmask indicating the positions of odd quotes.
/// * `escaped_quotes` - A bitmask indicating the positions of escaped quotes.
/// * `in_string` - A bitmask showing which characters are currently within a single-quoted string.
///
/// # Default Implementation
///
/// The `Default` implementation for `YamlSingleQuoteChunk` initializes
/// the fields `odd_quotes`, `escaped_quotes`, and `in_string` to 0.
///
/// ```rust
/// use yam_dark_core::YamlSingleQuoteChunk;
/// let y = YamlSingleQuoteChunk::default();
/// assert_eq!(y.odd_quotes, 0);
/// assert_eq!(y.escaped_quotes, 0);
/// assert_eq!(y.in_string, 0);
/// ```
pub struct YamlSingleQuoteChunk {
    /// Finds group of paired quotes
    pub odd_quotes: u64,

    /// Finds group of paired quotes like `''` or `''''` that are escaped.
    pub escaped_quotes: u64,

    /// Bitmask showing which characters are in string
    pub in_string: u64,
}

#[derive(Default)]
/// Represents the state of general character processing in YAML parsing.
///
/// `YamlCharacterChunk` is used to track the state of various character types such as whitespace,
/// line feeds, spaces, and structural characters within a YAML document.
///
/// # Fields
///
/// * `whitespace` - A bitmask indicating the positions of whitespace characters (`SPACE`, `TAB`, `LINE_FEED`, or `CARRIAGE_RETURN`).
/// * `spaces` - A bitmask indicating the positions of `SPACE` (`0x20`) characters.
/// * `line_feeds` - A bitmask indicating the positions of `LINE_FEED` (`0x0A`) characters.
/// * `structurals` - A bitmask indicating the positions of characters used as operators in YAML.
///
/// # Default Implementation
///
/// The `Default` implementation for `YamlCharacterChunk` initializes the fields `whitespace`,
/// `spaces`, `line_feeds`, and `structurals` to 0.
///
/// ```rust
/// use yam_dark_core::YamlCharacterChunk;
/// let y = YamlCharacterChunk::default();
/// assert_eq!(y.whitespace, 0);
/// assert_eq!(y.spaces, 0);
/// assert_eq!(y.line_feeds, 0);
/// assert_eq!(y.block_structurals, 0);
/// ```
pub struct YamlCharacterChunk {
    /// Whitespace bitmask SPACE  (`0x20`) , TABS (`0x09`), LINE_FEED (`0x0A`) or CARRIAGE_RETURN (`0x0D`)
    pub whitespace: u64,
    /// SPACE (`0x20`) bitmask
    pub spaces: u64,
    /// LINE_FEED (`0x0A`) bitmask
    pub line_feeds: u64,
    /// Block operators used in YAML
    pub block_structurals: u64,
    /// Flow operators used in YAML
    pub flow_structurals: u64,
    /// Bitmask showing if chunk character is in_comment
    pub in_comment: u64,
}

impl YamlCharacterChunk {
    #[must_use]
    pub const fn all_structurals(&self) -> u64 {
        self.flow_structurals | self.block_structurals
    }
}
#[test]
fn test_single_quotes1() {
    let mut block_state = YamlChunkState::default();
    let mut prev_iter_state = YamlParserState::default();

    let chunk = b" ' ''  '''                                                      ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.scan_single_quote_bitmask(&mut block_state, &mut prev_iter_state);
    let expected = 0b000000000000000000000000000000000000000000000000001110000010;
    assert_eq!(
        expected, block_state.single_quote.odd_quotes,
        "Expected:    {:#066b} \nGot instead: {:#066b} ",
        expected, block_state.single_quote.odd_quotes
    );
}

#[test]
fn test_single_quotes2() {
    let mut block_state = YamlChunkState::default();
    let mut prev_iter_state = YamlParserState::default();

    let chunk = b" ' ''  '' '                                                     ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.scan_single_quote_bitmask(&mut block_state, &mut prev_iter_state);
    let expected = 0b0000000000000000000000000000000000000000000000000010000000010;
    assert_eq!(
        expected, block_state.single_quote.odd_quotes,
        "Expected:    {:#066b} \nGot instead: {:#066b} ",
        expected, block_state.single_quote.odd_quotes
    );
}

#[test]
fn test_structurals() {
    let mut block_state = YamlChunkState::default();
    let chunk = b" -                                                              ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.classify_yaml_characters(&mut block_state);
    let expected = 0b000000000000000000000000000000000000000000000000000000000010;
    assert_eq!(
        block_state.characters.block_structurals, expected,
        "Expected:    {:#066b} \nGot instead: {:#066b} ",
        expected, block_state.single_quote.escaped_quotes
    );
}

#[test]
fn test_lteq() {
    let bin_str = b"                                                                ";
    let scanner = NativeScanner::from_chunk(bin_str);
    let result = scanner.unsigned_lteq_against_splat(0x20);
    assert_eq!(
        result,
        0b1111111111111111111111111111111111111111111111111111111111111111
    );
}

#[test]
fn test_count() {
    let bin_str = b"           \n                                                    ";
    let mut chunk = YamlChunkState::default();

    let expected_cols = (0..12).chain(0..52).collect::<Vec<_>>();
    let mut expected_row = vec![0; 12];
    expected_row.extend_from_slice(&[1; 52]);

    // Needs to be called before calculate indent
    let space_mask = u8x64_eq(bin_str, b' ');
    let newline_mask = u8x64_eq(bin_str, b'\n');
    // NativeScanner::calculate_cols_rows(&mut chunk.cols, &mut chunk.rows, newline_mask);
    assert_eq!(chunk.cols, expected_cols);
    assert_eq!(chunk.rows, expected_row);

    let expected_indents = vec![11, 52];
    NativeScanner::calculate_indents(&mut chunk.indents, newline_mask, space_mask, &mut true);
    assert_eq!(chunk.indents, expected_indents);
}

#[test]
fn test_count2() {
    let bin_str = b"    ab     \n          c                   \n                     ";
    let mut chunk = YamlChunkState::default();

    let cols = (0..12).chain(0..31).chain(0..21).collect::<Vec<_>>();

    let mut rows = vec![0; 12];
    rows.extend_from_slice(&[1; 31]);
    rows.extend_from_slice(&[2; 21]);

    let actual_indents = [4, 10, 21]; // or another default/expected value

    // Needs to be called before calculate indent
    let space_mask = u8x64_eq(bin_str, b' ');
    let newline_mask = u8x64_eq(bin_str, b'\n');

    NativeScanner::calculate_cols_rows(&mut chunk.cols, &mut chunk.rows, newline_mask);
    NativeScanner::calculate_indents(&mut chunk.indents, newline_mask, space_mask, &mut true);

    assert_eq!(chunk.cols, cols);
    assert_eq!(chunk.rows, rows);
    assert_eq!(chunk.indents, actual_indents);
}
