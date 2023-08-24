extern crate steel_yaml;

use std::fmt::Write;

use steel_yaml::tokenizer::{Event, EventIterator, StrReader};

pub fn assert_eq_event(input: &str, events: &str) {
    let mut line = String::new();
    let scan: EventIterator<StrReader> = EventIterator::from(input);
    for ev in scan {
        line.push_str("\n");
        write!(line, "{:}", ev).unwrap();
        if matches!(ev, Event::ErrorEvent) {
            break;
        }
    }

    assert_eq!(line, events, "Error in {input}");
}

