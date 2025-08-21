use std::str::from_utf8_unchecked;
use yam_common::Mark;
use yam_dark_core::{run_tape_to_end, EventListener, YamlParserState};

/// Struct used for testing equality of events.
pub struct StringTape {
    pub buff: String,
}

///
/// Assert that in for given input, the parser generates expected set of events
///
/// # Panics
///
///    Function panics if there is a difference between expected events string and one generated
///    from the input.
pub fn assert_eq_dark_event(input: &str, events: &str) {
    let mut event_tape = StringTape {
        buff: String::new(),
    };
    fill_string_tape(input, &mut event_tape);
    assert_eq!(event_tape.buff, events);
}

fn fill_string_tape(input: &str, event_tape: &mut StringTape) {
    let mut state = YamlParserState::default();

    if let Err(ref _e) = run_tape_to_end(input, &mut state, event_tape) {
        event_tape.buff.push_str("\nERR")
    }
}

impl EventListener for StringTape {
    type Value<'a> = &'a [u8];

    fn on_doc_start(&mut self) {
        self.buff.push_str("\nDOC");
    }

    fn on_scalar(&mut self, value: &[u8], mark: Mark) {
        self.buff.push_str("\n=VAL ");
        self.buff.push_str(unsafe { from_utf8_unchecked(value) });
    }

    fn on_scalar_owned(&mut self, value: Vec<u8>) {
        self.buff.push_str("\n=VAL ");
        unsafe { self.buff.as_mut_vec().extend(value) }
    }
}
