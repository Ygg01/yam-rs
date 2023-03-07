use crate::{tokenizer::StrReader, Spanner};

use super::{YamlToken, YamlTokenError};

pub struct YamlParser<'a> {
    pub(crate) spanner: Spanner,
    pub(crate) reader: StrReader<'a>,
    // pub(crate) map: HashMap<str,>,
}
