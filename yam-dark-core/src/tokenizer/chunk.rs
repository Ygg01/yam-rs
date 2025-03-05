#[allow(unused_imports)] // imports are used in tests
use crate::{u8x64_eq, NativeScanner, Stage1Scanner, YamlParserState};

/// Represents the state of YAML chunk processing.
///
/// `YamlChunkState` is used to track the state of various byte chunks,
/// including double-quoted strings, single-quoted strings, and character classifications
/// such as whitespace and structural characters.
///
/// This struct also maintains vectors for row and column positions and
/// indent levels, which are updated as the YAML content is processed.
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
    /// Basic `YamlChunkState` constructor, takes all important values and returns a valid `YamlChunkState`
    ///
    /// # Arguments
    ///
    /// * `single_quote`: Single quotes bitmask [`YamlSingleQuoteChunk`]
    /// * `double_quote`: Double quotes bitmask [`YamlDoubleQuoteChunk`]
    /// * `characters`: Other character bitmask [`YamlCharacterChunk`]
    ///
    ///
    /// # Examples
    ///
    /// ```rust
    /// use yam_dark_core::{YamlCharacterChunk, YamlChunkState, YamlDoubleQuoteChunk, YamlSingleQuoteChunk};
    ///
    /// let single_quote = YamlSingleQuoteChunk::default();
    /// let double_quote = YamlDoubleQuoteChunk::default();
    /// let characters = YamlCharacterChunk::default();
    ///
    /// let yaml_chunk_state = YamlChunkState::new_from_parts(single_quote, double_quote, characters);
    /// ```
    #[must_use]
    pub fn new_from_parts(
        single_quote: YamlSingleQuoteChunk,
        double_quote: YamlDoubleQuoteChunk,
        characters: YamlCharacterChunk,
    ) -> Self {
        YamlChunkState {
            double_quote,
            single_quote,
            characters,
            error_mask: 0,
        }
    }
}

impl YamlChunkState {
    /// Returns a [`u64`] where 1-bit, at given position, represents either flow or block
    /// structurals in the `[u8; 64]` chunk at corresponding position.
    #[must_use]
    pub const fn substructure(&self) -> u64 {
        self.characters.substructure()
            | self.double_quote.quote_starts
            | self.single_quote.quote_starts
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

    /// Bitmask indicating error in double quotes
    pub error_mask: u64,
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
/// assert_eq!(y.quote_starts, 0);
/// assert_eq!(y.escaped_quotes, 0);
/// assert_eq!(y.in_string, 0);
/// ```
pub struct YamlSingleQuoteChunk {
    /// Finds groups of start and end quotes
    pub quote_bits: u64,

    /// Finds groups of paired quotes like `''` or `''''` that are escaped.
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

    /// Possible unquoted scalars starts
    pub unquoted_scalars_starts: u64,

    /// Bits showing which bytes are in unquoted scalars.
    pub in_unquoted_scalars: u64,

    /// Bitmask showing if chunk character is in_comment
    pub in_comment: u64,
}

impl YamlCharacterChunk {
    #[must_use]
    /// Returns a [`u64`] where 1-bit, at given position, represents possible flow, block,
    /// or an unquoted start structurals character (a character) in the `[u8; 64]` chunk
    /// at corresponding position.
    pub const fn substructure(&self) -> u64 {
        self.unquoted_scalars_starts | self.block_structurals | self.flow_structurals
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::stage1::Stage1Scanner;
    use crate::util::str_to_chunk;
    use crate::{assert_bin_eq, NativeScanner, YamlParserState};
    use rstest::rstest;

    #[rstest]
    #[case(
        " ' ''  '''",
        0b00_0000_0010,
        0b10_0000_0010,
        0b01_1111_1110,
        0b01_1001_1000
    )]
    #[case(
        " ' ''  '' '",
        0b0000_0000_0010,
        0b0100_0000_0010,
        0b0011_1111_1110,
        0b0001_1001_1000
    )]
    #[case(
        "''' ''''' ",
        0b0_0000_0001,
        0b1_0000_0001,
        0b0_1111_1111,
        0b0_1111_0110
    )]
    fn test_single_quote(
        #[case] str: &str,
        #[case] quote_starts: u64,
        #[case] quote_bits: u64,
        #[case] in_string: u64,
        #[case] escaped: u64,
    ) {
        let scanner = NativeScanner::from_chunk(&str_to_chunk(str));
        let single_quote = scanner.scan_single_quote_bitmask(&mut YamlParserState::default());

        assert_bin_eq!(quote_starts, single_quote.quote_starts);
        assert_bin_eq!(quote_bits, single_quote.quote_bits);
        assert_bin_eq!(in_string, single_quote.in_string);
        assert_bin_eq!(escaped, single_quote.escaped_quotes);
    }

    #[rstest]
    #[case(
        " \"text with \\\"quote\\\" inside \"",
        0b10,
        0b10_0000_0000_0000_0000_0000_0000_0010,
        0b01_1111_1111_1111_1111_1111_1111_1110,
        0b00_0000_0001_0000_0010_0000_0000_0000
    )]
    fn test_double_quote(
        #[case] str: &str,
        #[case] quote_starts: u64,
        #[case] quote_bits: u64,
        #[case] in_string: u64,
        #[case] escaped: u64,
    ) {
        let scanner = NativeScanner::from_chunk(&str_to_chunk(str));
        let double_quote = scanner.scan_double_quote_bitmask(&mut YamlParserState::default());

        assert_bin_eq!(quote_starts, double_quote.quote_starts);
        assert_bin_eq!(quote_bits, double_quote.quote_bits);
        assert_bin_eq!(in_string, double_quote.in_string);
        assert_bin_eq!(escaped, double_quote.escaped);
    }

    #[rstest]
    fn test_lteq() {
        let bin_str = b"                                                                ";
        let scanner = NativeScanner::from_chunk(bin_str);
        let result = scanner.unsigned_lteq_against_splat(0x20);
        assert_bin_eq!(
            result,
            0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111
        );
    }

    #[rstest]
    fn test_structurals() {
        let chunk = b" -                                                              ";
        let scanner = NativeScanner::from_chunk(chunk);
        let characters = scanner.classify_yaml_characters();
        let expected =
            0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010;

        assert_bin_eq!(characters.block_structurals, expected);
    }

    #[test]
    fn test_scan_single_quote_bitmask() {
        let mut prev_iter_state = YamlParserState::default();

        let chunk = b" ' ''  '' '                                                     ";
        let scanner = NativeScanner::from_chunk(chunk);
        let single_quote = scanner.scan_single_quote_bitmask(&mut prev_iter_state);
        assert_bin_eq!(0b0000_0000_0010, single_quote.quote_starts);
        assert_bin_eq!(0b0100_0000_0010, single_quote.quote_bits);
        assert_bin_eq!(0b0011_1111_1110, single_quote.in_string);
        assert_bin_eq!(0b0001_1001_1000, single_quote.escaped_quotes);
    }

    #[test]
    fn test_unquoted_start() {
        let string = " - a  b \n - a  b ";
        let scanner = NativeScanner::from_chunk(&str_to_chunk(string));

        let character_chunk = scanner.classify_yaml_characters();
        let structure_bit = character_chunk.substructure();

        assert_bin_eq!(0b0001_0100_0000_1010, structure_bit);
        assert_bin_eq!(0b1111_0000_0111_1000, character_chunk.in_unquoted_scalars);
    }
}
