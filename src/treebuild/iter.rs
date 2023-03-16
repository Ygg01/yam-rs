use std::collections::HashMap;

use crate::{tokenizer::StrReader, Lexer};

use super::{YamlToken};
pub struct YamlParser<'a, TAG> {
    pub(crate) spanner: Lexer,
    pub(crate) reader: StrReader<'a>,
    pub(crate) map: HashMap<String, YamlToken<'a, TAG>>,
}
