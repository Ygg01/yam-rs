use std::collections::HashMap;
use std::marker::PhantomData;

use crate::Lexer;
use crate::tokenizer::Reader;

use super::YamlToken;

pub struct YamlParser<'a, R, B, TAG> {
    pub(crate) lexer: Lexer,
    pub(crate) reader: R,
    pub(crate) map: HashMap<String, YamlToken<'a, TAG>>,
    buf: PhantomData<B>,
}

impl<'a, R, B, TAG> From<&'a str> for YamlParser<'a, R, B, TAG> where R: Reader<()> + From<&'a str> {
    fn from(value: &'a str) -> Self {
        YamlParser {
            lexer: Lexer::default(),
            reader: From::from(value),
            map: HashMap::default(),
            buf: PhantomData::default(),
        }
    }
}

impl<'a, R, B, TAG> From<R> for YamlParser<'a, R, B, TAG> where R: Reader<()> + From<R> {
    fn from(value: R) -> Self {
        YamlParser {
            lexer: Lexer::default(),
            reader: value,
            map: HashMap::default(),
            buf: PhantomData::default(),
        }
    }
}