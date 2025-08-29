#[allow(unused_imports)] // imports are used in tests
use crate::{u8x64_eq, NativeScanner, Stage1Scanner};

/// Represents the state of YAML chunk processing.
///
/// `YamlChunkState` is used to track the state of various byte chunks,
/// including double-quoted strings, single-quoted strings, and character classifications
/// such as whitespace and structural characters.
pub struct YamlChunkState {
    /// [`YamlDoubleQuoteChunk`] struct containing double-quoted YAML strings information.
    pub double_quote: YamlDoubleQuoteChunk,
    /// [`YamlSingleQuoteChunk`] struct containing single-quoted YAML strings information.
    pub single_quote: YamlSingleQuoteChunk,
    /// [`YamlCharacterChunk`] struct containing info for characters (e.g., whitespace, operators).
    pub characters: YamlCharacterChunk,
    /// Last character
    pub last_char: u8,
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
            last_char: 0,
        }
    }
}

impl YamlChunkState {
    /// Returns a [`u64`] where 1-bit, at a given position, represents either flow or block
    /// structurals in the `[u8; 64]` chunk at a corresponding position.
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
/// assert_eq!(y.in_string, 0);
/// ```
pub struct YamlDoubleQuoteChunk {
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
/// assert_eq!(y.in_string, 0);
/// ```
pub struct YamlSingleQuoteChunk {
    /// Finds groups of start and end quotes
    pub quote_bits: u64,

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
    /// Whitespace bitmask `SPACE` (`0x20`) , `TABS` (`0x09`), `LINE_FEED` (`0x0A`) or `CARRIAGE_RETURN` (`0x0D`)
    pub whitespace: u64,

    /// `SPACE` (`0x20`) bitmask
    pub spaces: u64,

    /// `LINE_FEED` (`0x0A`) bitmask
    pub line_feeds: u64,

    /// Block operators used in YAML
    pub block_structurals: u64,

    /// Flow operators used in YAML
    pub flow_structurals: u64,

    /// Possible unquoted scalars starts
    pub unquoted_scalars_starts: u64,

    /// Bits showing which bytes are in unquoted scalars.
    pub in_unquoted_scalars: u64,

    /// Bitmask showing if chunk character is `in_comment`
    pub in_comment: u64,

    /// Bitmask showing comment start
    pub comment_start: u64,
}

impl YamlCharacterChunk {
    #[must_use]
    /// Returns a [`u64`] where 1-bit, at a given position, represents possible flow, block,
    /// or an unquoted start structurals character (a character) in the `[u8; 64]` chunk
    /// at a corresponding position.
    pub const fn substructure(&self) -> u64 {
        self.unquoted_scalars_starts | self.block_structurals | self.flow_structurals
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::parser::ChunkIterState;
    use crate::tokenizer::stage1::Stage1Scanner;
    use crate::tokenizer::YamlParserState;
    use crate::util::str_to_chunk;
    use crate::{assert_bin_eq, NativeScanner};
    use alloc::vec;
    use alloc::vec::Vec;
    use rstest::rstest;

    #[rstest]
    #[case(" ' ''  '''", 0b00_0000_0010, 0b10_0000_0010, 0b01_1111_1110)]
    #[case(" ' ''  '' '", 0b0000_0000_0010, 0b0100_0000_0010, 0b0011_1111_1110)]
    #[case("''' ''''' ", 0b0_0000_0001, 0b1_0000_0001, 0b0_1111_1111)]
    fn test_single_quote(
        #[case] str: &str,
        #[case] quote_starts: u64,
        #[case] quote_bits: u64,
        #[case] in_string: u64,
    ) {
        let scanner = NativeScanner::from_chunk(&str_to_chunk(str));
        let single_quote = scanner.scan_single_quote_bitmask(
            &mut ChunkIterState::default(),
            &mut YamlParserState::default(),
        );

        assert_bin_eq!(quote_starts, single_quote.quote_starts);
        assert_bin_eq!(quote_bits, single_quote.quote_bits);
        assert_bin_eq!(in_string, single_quote.in_string);
    }

    #[rstest]
    #[case(
        " \"text with \\\"quote\\\" inside \"",
        0b10,
        0b01_1111_1111_1111_1111_1111_1111_1110
    )]
    fn test_double_quote(#[case] str: &str, #[case] quote_starts: u64, #[case] in_string: u64) {
        let scanner = NativeScanner::from_chunk(&str_to_chunk(str));
        let double_quote = scanner.scan_double_quote_bitmask(
            &mut ChunkIterState::default(),
            &mut YamlParserState::default(),
        );

        assert_bin_eq!(quote_starts, double_quote.quote_starts);
        assert_bin_eq!(in_string, double_quote.in_string);
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
        let mut chunk_iter_state = ChunkIterState::default();

        let chunk = "''' ''''' ";
        let scanner = NativeScanner::from_chunk(&str_to_chunk(chunk));
        let single_quote =
            scanner.scan_single_quote_bitmask(&mut chunk_iter_state, &mut prev_iter_state);
        assert_bin_eq!(0b0000_0000_0001, single_quote.quote_starts);
        assert_bin_eq!(0b0001_0000_0001, single_quote.quote_bits);
        assert_bin_eq!(0b0000_1111_1111, single_quote.in_string);
    }

    #[test]
    fn test_unquoted_start() {
        let string = " - a  b";
        let chunk = str_to_chunk(string);
        let mut state = YamlParserState::default();
        let mut chunk_iter_state = ChunkIterState::default();

        let chunk = NativeScanner::next(&chunk, &mut chunk_iter_state, &mut state, &mut 0);
        let structure_bit = chunk.substructure();

        assert_bin_eq!(0b0000_1010, structure_bit);
        assert_bin_eq!(0b0111_1000, chunk.characters.in_unquoted_scalars);
    }

    struct ArrayPattern {
        pub patterns: &'static [(u32, u8)],
    }

    impl ArrayPattern {
        fn into_array(self) -> [u32; 64] {
            let mut vec = Vec::with_capacity(64);
            for pattern in self.patterns {
                let patter_vec = vec![pattern.0; pattern.1 as usize];
                vec.extend(patter_vec);
            }
            assert_eq!(vec.len(), 64, "Expected 64 elements, got: {}", vec.len());
            vec.as_slice().try_into().unwrap()
        }
    }

    const CASE_AB_PATTERN: ArrayPattern = ArrayPattern {
        patterns: &[(69, 8), (2, 56)],
    };

    const CASE_AB_NO_CONT_PATTERN: ArrayPattern = ArrayPattern {
        patterns: &[(5, 8), (2, 56)],
    };

    const NO_INDENT_PATTERN: ArrayPattern = ArrayPattern {
        patterns: &[(0, 3), (2, 61)],
    };

    #[rstest]
    #[case("     a \n  b", true, 0, CASE_AB_NO_CONT_PATTERN)]
    #[case("     a \n  b", true, 64, CASE_AB_PATTERN)]
    #[case("a \n  b", true, 0, NO_INDENT_PATTERN)]
    fn test_calculate_relative(
        #[case] string: &str,
        #[case] is_indent_running: bool,
        #[case] previous_indent: u32,
        #[case] expected: ArrayPattern,
    ) {
        let scanner = NativeScanner::from_chunk(&str_to_chunk(string));
        let mut parse_state = YamlParserState::default();
        let mut chunk_iter_state = ChunkIterState {
            previous_indent,
            is_indent_running,
            ..Default::default()
        };

        let character_chunk = scanner.classify_yaml_characters();
        let mut indents = [0; 64];
        NativeScanner::calculate_relative_indents(
            &character_chunk,
            &mut chunk_iter_state,
            &mut parse_state,
            &mut indents,
        );
        assert_eq!(indents, expected.into_array());
    }

    #[test]
    fn test_calculate_relative_individual() {
        let string = "a \n  b";
        let is_indent_running = true;
        let previous_indent = 0;
        let expected = NO_INDENT_PATTERN.into_array();

        let scanner = NativeScanner::from_chunk(&str_to_chunk(string));
        let mut parse_state = YamlParserState::default();
        let mut chunk_iter_state = ChunkIterState {
            previous_indent,
            is_indent_running,
            ..Default::default()
        };

        let character_chunk = scanner.classify_yaml_characters();
        let mut indents = [0; 64];
        NativeScanner::calculate_relative_indents(
            &character_chunk,
            &mut chunk_iter_state,
            &mut parse_state,
            &mut indents,
        );

        assert_eq!(indents, expected);
    }
}
