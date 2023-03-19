use std::borrow::Cow;
use std::borrow::Cow::Borrowed;
use std::collections::{HashMap, VecDeque};
use std::marker::PhantomData;
use std::mem::take;

use crate::tokenizer::LexerToken::*;
use crate::tokenizer::{ErrorType, LexerToken, Reader, Slicer};
use crate::treebuild::Entry;
use crate::Lexer;

use super::YamlToken;

pub struct YamlParser<'a, R, B = (), TAG = ()> {
    pub(crate) reader: R,
    pub(crate) map: HashMap<String, YamlToken<'a, TAG>>,
    buf: PhantomData<B>,
}

impl<'a, R, B, TAG: Default> YamlParser<'a, R, B, TAG>
where
    R: Reader<B> + Slicer<'a>,
{
    pub fn parse_doc(&'a mut self) -> YamlToken<'a, TAG> {
        let mut lexer = Lexer::default();
        while !lexer.stream_end {
            lexer.fetch_next_token(&mut self.reader);
        }
        let mut val = YamlToken::default();
        let mut tag = TAG::default();
        while !lexer.tokens.is_empty() {
            if let Some(x) = lexer.tokens.pop_front() {
                let token = x.into();
                match token {
                    SequenceStart => {
                        val = self.parse_sequence(
                            &mut lexer.tokens,
                            &mut lexer.errors,
                            take(&mut tag),
                        );
                        break;
                    }
                    MappingStart => {
                        val = self.parse_mapping(
                            &mut lexer.tokens,
                            &mut lexer.errors,
                            take(&mut tag),
                        );
                        break;
                    }
                    ScalarPlain | ScalarFold | ScalarLit | ScalarSingleQuote
                    | ScalarDoubleQuote => {
                        val =
                            self.parse_scalar(&mut lexer.tokens, &mut lexer.errors, take(&mut tag));
                        break;
                    }

                    _ => {}
                }
            }
        }
        val
    }

    fn parse_sequence(
        &'a self,
        tokens: &mut VecDeque<usize>,
        _errors: &mut Vec<ErrorType>,
        seq_tag: TAG,
    ) -> YamlToken<'a, TAG> {
        let mut seq_value = vec![];
        let mut tag = TAG::default();
        while let Some(x) = tokens.pop_front() {
            let token: LexerToken = x.into();
            match token {
                TagStart => tag = self.parse_tag(),
                SequenceEnd => break,
                SequenceStart => {
                    seq_value.push(self.parse_sequence(tokens, _errors, take(&mut tag)))
                }
                MappingStart => {
                    seq_value.push(self.parse_mapping(tokens, _errors, take(&mut tag)));
                }
                _ => {}
            }
        }
        YamlToken::Sequence(seq_value, seq_tag)
    }
    fn parse_tag(&self) -> TAG {
        TAG::default()
    }
    fn parse_mapping(
        &'a self,
        tokens: &mut VecDeque<usize>,
        _errors: &mut Vec<ErrorType>,
        map_tag: TAG,
    ) -> YamlToken<TAG> {
        let mut seq_value = vec![];
        let mut tag = TAG::default();
        let mut entry = Entry::default();
        let mut is_key = true;
        while let Some(x) = tokens.pop_front() {
            let token: LexerToken = x.into();
            match token {
                TagStart => tag = self.parse_tag(),
                MappingEnd => break,
                SequenceStart if is_key => {
                    is_key = false;
                    entry.key = self.parse_sequence(tokens, _errors, take(&mut tag));
                }
                SequenceStart if !is_key => {
                    is_key = true;
                    entry.value = self.parse_sequence(tokens, _errors, take(&mut tag));
                    seq_value.push(take(&mut entry));
                }
                MappingStart if is_key => {
                    is_key = false;
                    entry.key = self.parse_mapping(tokens, _errors, take(&mut tag));
                }
                MappingStart if !is_key => {
                    is_key = true;
                    entry.value = self.parse_mapping(tokens, _errors, take(&mut tag));
                    seq_value.push(take(&mut entry));
                }
                ScalarPlain | ScalarFold | ScalarLit | ScalarSingleQuote | ScalarDoubleQuote
                    if is_key =>
                {
                    is_key = false;
                    entry.key = self.parse_scalar(tokens, _errors, take(&mut tag));
                }
                ScalarPlain | ScalarFold | ScalarLit | ScalarSingleQuote | ScalarDoubleQuote
                    if !is_key =>
                {
                    is_key = true;
                    entry.value = self.parse_scalar(tokens, _errors, take(&mut tag));
                    seq_value.push(take(&mut entry));
                }
                _ => {}
            }
        }
        YamlToken::Mapping(seq_value, map_tag)
    }

    fn parse_scalar(
        &'a self,
        tokens: &mut VecDeque<usize>,
        _errors: &mut Vec<ErrorType>,
        scalar_tag: TAG,
    ) -> YamlToken<TAG> {
        let mut cow: Cow<'a, [u8]> = Cow::default();
        loop {
            match (tokens.get(0), tokens.get(1)) {
                (Some(start), Some(end))
                    if *start < NewLine as usize && *end < NewLine as usize =>
                {
                    if cow.is_empty() {
                        cow = Borrowed(self.reader.slice(*start, *end))
                    } else {
                        cow.to_mut().extend(self.reader.slice(*start, *end))
                    }
                    tokens.pop_front();
                    tokens.pop_front();
                }
                (Some(newline), Some(line)) if *newline == NewLine as usize => {
                    if *line == 0 {
                        cow.to_mut().extend(" ".as_bytes());
                    } else {
                        cow.to_mut().extend("\n".repeat(*line).as_bytes())
                    }
                    tokens.pop_front();
                    tokens.pop_front();
                }
                (_, _) => break,
            }
        }

        YamlToken::Scalar(cow, scalar_tag)
    }
}

impl<'a, R, B, TAG> From<&'a str> for YamlParser<'a, R, B, TAG>
where
    R: Reader<()> + From<&'a str>,
{
    fn from(value: &'a str) -> Self {
        YamlParser {
            reader: From::from(value),
            map: HashMap::default(),
            buf: PhantomData::default(),
        }
    }
}

impl<'a, R, B, TAG> From<R> for YamlParser<'a, R, B, TAG>
where
    R: Reader<()> + From<R>,
{
    fn from(value: R) -> Self {
        YamlParser {
            reader: value,
            map: HashMap::default(),
            buf: PhantomData::default(),
        }
    }
}
