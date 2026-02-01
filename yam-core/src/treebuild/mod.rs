use crate::saphyr_tokenizer::SpannedEventReceiver;
use crate::saphyr_tokenizer::{Event, Source, StrSource};
use crate::{Parser, Span};
use alloc::vec::Vec;
use yam_common::{YamlDoc, YamlError};

#[derive(Default)]
pub struct YamlLoader<'input> {
    docs: Vec<YamlDoc<'input>>,
}

impl<'input> SpannedEventReceiver<'input> for YamlLoader<'input> {
    fn on_event(&mut self, ev: Event<'input>, span: Span) {
        todo!()
    }
}

impl YamlLoader<'_> {
    pub fn load_parser<'a, I: Source>(
        parser: &mut Parser<'a, I>,
    ) -> Result<Vec<YamlDoc<'a>>, YamlError> {
        let mut event_listener = YamlLoader::default();
        parser.load(&mut event_listener, true)?;
        Ok(event_listener.docs)
    }

    pub fn load_from<'a, S: AsRef<str>>(input: S) -> Result<Vec<YamlDoc<'a>>, YamlError> {
        let mut event_listener = YamlLoader::default();
        let mut parser = Parser::new(StrSource::new(input.as_ref()));
        parser.load(&mut event_listener, true)?;
        Ok(event_listener.docs)
    }
}
