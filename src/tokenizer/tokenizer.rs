use crate::tokenizer::reader::{Reader, StrReader};
use crate::tokenizer::{StrIterator, YamlToken};
use crate::tokenizer::tokenizer::SpanToken::SeqStart;
use crate::tokenizer::tokenizer::State::StreamStart;

#[derive(Clone, Default)]
pub struct YamlTokenizer {
    state: State,
}

#[derive(Copy, Clone)]
pub enum State {
    StreamStart,
    Post
}

impl Default for State {
    fn default() -> Self {
        State::StreamStart
    }
}

pub enum SpanToken {
    Scalar(usize, usize),
    SeqStart,
}

impl YamlTokenizer {
    pub(crate) fn read_token<T: Reader>(&mut self, reader: &mut T) -> Option<SpanToken> {
        match self.state {
            StreamStart => self.read_stream(reader),
            _ => None,
        }
    }

    pub(crate) fn read_stream<T: Reader>(&mut self, reader: &mut T) -> Option<SpanToken> {
        // TODO BOM dealing
        if let Some(x) =  self.try_read_comment(reader) {
            return Some(x);
        }

        return None;
    }

    pub fn from_string(self, slice: &str) -> StrIterator {
        StrIterator {
            state: self,
            reader: StrReader::new(slice),
        }
    }
    fn try_read_comment<T: Reader>(&self, reader: &mut T) -> Option<SpanToken> {
        reader.skip_space_tab();

        if reader.try_read_slice_exact("#") {
            reader.read_fast_until(&[])
        }

        None
    }
}
