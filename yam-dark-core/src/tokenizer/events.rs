#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Stage1Discriminant {
    /// Directive Tag denoted by `%TAG`
    DirectiveTag = b'%',
    /// Plain unquoted scalar that's neither quoted or literal or folded
    /// ```yaml
    ///     example: plain_scalar
    /// ```
    ScalarPlain = b's',
    /// Helper token to end token
    /// Folded scalar token
    /// ```yaml
    ///     example: >
    ///         folded_scalar
    /// ```
    ScalarFold = b'>',
    /// Literal scalar token
    /// ```yaml
    ///     example: |
    ///         literal_scalar
    /// ```
    ScalarLit = b'|',
    /// Single quoted scalar
    /// ```yaml
    ///     example: 'single quote scalar'
    /// ```
    ScalarSingleQuote = b'\'',
    /// Double quoted scalar
    /// ```yaml
    ///     example: "double quote scalar"
    /// ```
    ScalarDoubleQuote = b'"',
    /// Element with alternative name e.g. `&foo [x,y]`
    AnchorToken = b'&',
    /// Reference to an element with alternative name e.g. `*foo`
    AliasToken = b'*',
    /// Tag token in YAML.
    /// ```yaml
    ///    !tag b: c
    ///   #^--- YAML tag
    /// ```
    Tag = b'!',
    /// Start of a sequence token, e.g. `[` in
    /// ```yaml
    ///  [ a, b, c]
    /// #^ - start of sequence
    /// ```
    SequenceStartExplicit = b'[' | 128,
    /// Start of a sequence token, e.g. `[` in
    /// ```yaml
    ///  [ a, b, c]
    /// #^ - start of sequence
    /// ```
    SequenceStart = b'[',
    /// End of a sequence token, e.g. `]` in
    /// ```yaml
    ///  [a, b, c]
    /// #        ^-- end of sequence
    /// ```
    SequenceEnd = b']',
    /// Start of a map  token, e.g. `{` in
    /// ```yaml
    ///  { a: b,}
    /// #^ - start of mapping
    /// ```
    MappingStartExplicit = b'{' | 128,
    /// Start of a map  token, e.g. `{` in
    /// ```yaml
    ///  [ a]: 3
    /// #^ - start of mapping
    /// ```
    MappingStart = b'{',
    /// End of a map  token, e.g. `}` in
    /// ```yaml
    ///  { a: b }
    /// #       ^-- start of mapping
    /// ```
    MappingEnd = b'}',
    /// Limit  of implicit Document.
    DocumentStart = b'r',
    /// Limit of explicit Document.
    DocumentStartExplicit = b'r' | 128,
    /// Null/empty value
    Null = b'n',
    /// Double value
    Double = b'd',
    /// Long value
    Long = b'l',
    /// Unsigned long value
    UnsignedLong = b'u',
}