use std::collections::HashMap;

use crate::{tokenizer::StrReader, Spanner};

use super::{YamlToken, YamlTokenError};

pub struct YamlParser<'a, TAG> {
    pub(crate) spanner: Spanner,
    pub(crate) reader: StrReader<'a>,
    pub(crate) map: HashMap<String, YamlToken<'a, TAG>>,
}
