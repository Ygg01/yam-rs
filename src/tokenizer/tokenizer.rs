use crate::tokenizer::reader::Reader;
use crate::tokenizer::YamlToken;

/// Contains state change algorithm will
#[derive(Clone)]
pub struct YamlTokenizer {}

pub enum SpanToken {
    Scalar(usize, usize),
}

impl YamlTokenizer {
    pub(crate) fn read_token<T: Reader>(&mut self, reader: &mut T) -> Option<SpanToken> {
        todo!()
    }
}