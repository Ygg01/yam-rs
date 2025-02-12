#[allow(unused_imports)] // imports are used in tests
use crate::{u8x64_eq, NativeScanner, Stage1Scanner, YamlParserState};

#[derive(Default)]
/// Represents the state of YAML chunk processing.
///
/// `YamlChunkState` is used to track the state of various chunks of YAML content,
/// including double-quoted strings, single-quoted strings, and character classifications
/// such as whitespace and structural characters.
///
/// This struct also maintains vectors for row and column positions and
/// indent levels, which are updated as the YAML content is processed.
///
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
    /// [`YamlDoubleQuoteChunk`] struct containing double-quoted YAML strings information.
    pub double_quote: YamlDoubleQuoteChunk,
    /// [`YamlSingleQuoteChunk`] struct containing single-quoted YAML strings information.
    pub single_quote: YamlSingleQuoteChunk,
    /// [`YamlCharacterChunk`] struct containing info for characters (e.g., whitespace, operators).
    pub characters: YamlCharacterChunk,
    /// Bitmask indicating positions with errors
    pub(crate) error_mask: u64,
}

impl YamlChunkState {
    /// Returns a [`u64`] where 1-bit, at given position, represents either flow or block
    /// structurals in the `[u8; 64]` chunk at corresponding position.
    #[must_use]
    pub const fn all_structurals(&self) -> u64 {
        self.characters.flow_structurals | self.characters.block_structurals | self.double_quote.quote_starts | self.single_quote.quote_starts
    }
}
#[derive(Default)]
/// Represents the state of double-quoted YAML string processing.
///
/// `YamlDoubleQuoteChunk` is used to track the state of double-quoted YAML strings,
/// maintaining information about escaped characters, real double quotes, and whether
/// characters are within the string context.
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
    /// Bitmask indicating the positions of escaped characters.
    pub escaped: u64,

    /// Bitmask indicating the positions of real double quotes.
    pub quote_bits: u64,

    /// Bitmask showing which characters are currently within a double-quoted string.
    pub in_string: u64,

    /// Bitmask indicating the starts of double quotes.
    pub quote_starts: u64,
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
    pub quote_bits: u64,

    /// Finds group of paired quotes like `''` or `''''` that are escaped.
    pub escaped_quotes: u64,

    /// Bitmask showing which characters are in string
    pub in_string: u64,

    /// Bitmask indicating the starts of double quotes.
    pub quote_starts: u64,
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


#[test]
fn test_single_quotes1() {
    let mut block_state = YamlChunkState::default();
    let mut prev_iter_state = YamlParserState::default();

    let chunk = b" ' ''  '''                                                      ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.scan_single_quote_bitmask(&mut block_state, &mut prev_iter_state);
    let expected =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0011_1000_0010;
    assert_eq!(
        expected, block_state.single_quote.quote_bits,
        "Expected:    {:#066b} \nGot instead: {:#066b} ",
        expected, block_state.single_quote.quote_bits
    );
}

#[test]
fn test_single_quotes2() {
    let mut block_state = YamlChunkState::default();
    let mut prev_iter_state = YamlParserState::default();

    let chunk = b" ' ''  '' '                                                     ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.scan_single_quote_bitmask(&mut block_state, &mut prev_iter_state);
    let expected =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0010;

    assert_eq!(
        expected, block_state.single_quote.quote_bits,
        "Expected:    {:#066b} \nGot instead: {:#066b} ",
        expected, block_state.single_quote.quote_bits
    );
}

#[test]
fn test_structurals() {
    let mut block_state = YamlChunkState::default();
    let chunk = b" -                                                              ";
    let scanner = NativeScanner::from_chunk(chunk);
    scanner.classify_yaml_characters(&mut block_state);
    let expected =
        0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010;
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
        0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111
    );
}
