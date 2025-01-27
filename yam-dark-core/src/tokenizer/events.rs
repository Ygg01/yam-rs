const DOC_END: u8 = b'.';
const DOC_END_EXP: u8 = DOC_END ^ b'x';
const DOC_START: u8 = b'-';
const DOC_START_EXP: u8 = DOC_START ^ b'x';
const MAP_END: u8 = b'}';
const MAP_START_EXP: u8 = MAP_START ^ b'x';
const MAP_START: u8 = b'{';
const SEQ_END: u8 = b']';
const SEQ_START: u8 = b'[';
const SEQ_START_EXP: u8 = SEQ_START ^ b'x';
const SCALAR_PLAIN: u8 = b's';
const SCALAR_FOLD: u8 = b'>';
const SCALAR_LIT: u8 = b'|';
const SCALAR_QUOTE: u8 = b'\'';
const SCALAR_DQUOTE: u8 = b'"';
const TAG: u8 = b'!';
const ANCHOR: u8 = b'&';
const ALIAS: u8 = b'*';
const DIRECTIVE: u8 = b'%';
const NULL: u8 = b'n';
const DOUBLE: u8 = b'd';
const LONG: u8 = b'l';
const UNSIGNED_LONG: u8 = b'u';

#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Stage1TapeEvent {
    /// Directive Tag denoted by `%TAG`
    DirectiveTag = DIRECTIVE,
    /// Plain unquoted scalar that's neither quoted or literal or folded
    /// ```yaml
    ///     example: plain_scalar
    /// ```
    ScalarPlain = SCALAR_PLAIN,
    /// Helper token to end token
    /// Folded scalar token
    /// ```yaml
    ///     example: >
    ///         folded_scalar
    /// ```
    ScalarFold = SCALAR_FOLD,
    /// Literal scalar token
    /// ```yaml
    ///     example: |
    ///         literal_scalar
    /// ```
    ScalarLit = SCALAR_LIT,
    /// Single quoted scalar
    /// ```yaml
    ///     example: 'single quote scalar'
    /// ```
    ScalarSingleQuote = SCALAR_QUOTE,
    /// Double quoted scalar
    /// ```yaml
    ///     example: "double quote scalar"
    /// ```
    ScalarDoubleQuote = SCALAR_DQUOTE,
    /// Element with alternative name e.g. `&foo [x,y]`
    AnchorToken = ANCHOR,
    /// Reference to an element with alternative name e.g. `*foo`
    AliasToken = ALIAS,
    /// Tag
    Tag = TAG,
    /// Start of a sequence token, e.g. `[` in
    /// ```yaml
    ///  [ a, b, c]
    /// #^ - start of sequence
    /// ```
    SequenceStartExplicit = SEQ_START_EXP,
    /// Start of a sequence token, e.g. `[` in
    /// ```yaml
    ///  [ a, b, c]
    /// #^ - start of sequence
    /// ```
    SequenceStart = SEQ_START,
    /// End of a sequence token, e.g. `]` in
    /// ```yaml
    ///  [a, b, c]
    /// #        ^-- end of sequence
    /// ```
    SequenceEnd = SEQ_END,
    /// Start of a map  token, e.g. `{` in
    /// ```yaml
    ///  { a: b,}
    /// #^ - start of mapping
    /// ```
    MappingStartExplicit = MAP_START_EXP,
    /// Start of a map  token, e.g. `{` in
    /// ```yaml
    ///  [ a]: 3
    /// #^ - start of mapping
    /// ```
    MappingStart = MAP_START,
    /// End of a map  token, e.g. `}` in
    /// ```yaml
    ///  { a: b }
    /// #       ^-- start of mapping
    /// ```
    MappingEnd = MAP_END,
    /// Start of implicit Document
    DocumentStart = DOC_START,
    /// Start of explicit Document
    DocumentStartExplicit = DOC_START_EXP,
    /// End of implicit document.
    DocumentEnd = DOC_END,
    /// End of explicit document.
    DocumentEndExplicit = DOC_END_EXP,
    /// Null/empty value
    Null = NULL,
}