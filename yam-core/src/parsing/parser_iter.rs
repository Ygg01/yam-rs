use crate::parsing::{Event, Parser, ScalarValue, Source, StrSource, Tag};
use crate::prelude::YamlError;
use alloc::borrow::Cow;
use core::ops::ControlFlow;
use log::debug;

#[derive(Default, PartialEq, Eq, Clone, Copy)]
enum State {
    #[default]
    StreamStart,
    InDocument,
    Sequence,
    EndDocument,
}

pub enum YamEvent<'de> {
    DocStart,
    DocEnd,
    StreamEnd,
    Scalar(ScalarValue<'de>),
    SeqStart(usize, Option<Cow<'de, Tag>>),
    SeqEnd,
    MapStart(usize, Option<Cow<'de, Tag>>),
    MapEnd,
}

struct ParserIter<'de, R: Source> {
    parser: Parser<'de, R>,
    state: State,
}

impl<'a> ParserIter<'a, StrSource<'a>> {
    pub fn from_str<S: AsRef<str>>(input: &'a S) -> Self {
        Self::new(StrSource::new(input.as_ref()))
    }
}

impl<'de, R> ParserIter<'de, R>
where
    R: Source,
{
    fn new(input: R) -> Self {
        Self {
            parser: Parser::new(input),
            state: State::StreamStart,
        }
    }
}

impl<'de, R> Iterator for ParserIter<'de, R>
where
    R: Source,
{
    type Item = YamEvent<'de>;

    fn next(&mut self) -> Option<Self::Item> {
        let res = loop {
            match self.state {
                State::StreamStart => break self.process_start(),
                State::InDocument => match self.process_doc() {
                    ControlFlow::Break(flow) => break flow,
                    ControlFlow::Continue(()) => continue,
                },
                State::Sequence => {}
                State::EndDocument => break self.finish_document(),
            }
        };
        match res {
            Err(e) => {
                debug!("{e}");
                None
            }
            Ok(YamEvent::StreamEnd) => None,
            Ok(res) => Some(res),
        }
    }
}
impl<'de, R: Source> ParserIter<'de, R> {
    pub(crate) fn process_start(&mut self) -> Result<YamEvent<'de>, YamlError> {
        // Expect a <stream-start>
        let ev = self.parser.next_event_impl()?.0;
        if ev != Event::StreamStart {
            return Err(YamlError::new_custom("Expected Stream start"));
        }
        // Expect a <doc-start>
        let ev = self.parser.next_event_impl()?.0;
        if !ev.is_doc_start() {
            return Err(YamlError::new_custom("Expected Document start"));
        }
        self.state = State::InDocument;
        Ok(YamEvent::DocStart)
    }

    pub(crate) fn process_doc(&mut self) -> ControlFlow<Result<YamEvent<'de>, YamlError>> {
        let ev = match self.parser.next_event_impl() {
            Ok(ev) => ev.0,
            Err(err) => return ControlFlow::Break(Err(err)),
        };
        match ev {
            // Ignored events
            Event::Nothing | Event::Alias(_) | Event::Comment(_) => ControlFlow::Continue(()),
            // Unexpected events
            Event::StreamStart => {
                ControlFlow::Break(Err(YamlError::new_custom("Unexpected Stream start")))
            }
            Event::StreamEnd => {
                ControlFlow::Break(Err(YamlError::new_custom("Unexpected Stream end")))
            }
            Event::DocumentStart(_) => {
                ControlFlow::Break(Err(YamlError::new_custom("Unexpected Document start")))
            }

            Event::DocumentEnd => {
                self.state = State::EndDocument;
                ControlFlow::Break(Ok(YamEvent::DocEnd))
            }

            Event::Scalar(a) => ControlFlow::Break(Ok(YamEvent::Scalar(a))),
            Event::SequenceStart(alias, tag) => {
                ControlFlow::Break(Ok(YamEvent::SeqStart(alias, tag)))
            }
            Event::SequenceEnd => ControlFlow::Break(Ok(YamEvent::SeqEnd)),
            Event::MappingStart(alias, tag) => {
                ControlFlow::Break(Ok(YamEvent::MapStart(alias, tag)))
            }
            Event::MappingEnd => ControlFlow::Break(Ok(YamEvent::MapEnd)),
        }
    }

    pub(crate) fn finish_document(&mut self) -> Result<YamEvent<'de>, YamlError> {
        // Expect a <stream-start>
        let ev = self.parser.next_event_impl()?.0;
        if ev != Event::StreamEnd {
            return Err(YamlError::new_custom("Expected Stream start"));
        }

        Ok(YamEvent::StreamEnd)
    }
}
