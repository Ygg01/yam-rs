extern crate steel_yaml;

use std::fmt::Write;

use steel_yaml::tokenizer::{EventIterator, StrReader, Event};


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

pub fn assert_eq_event_exact(input: &str, events: &str) {
    let mut line = String::new();
    let scan: EventIterator<StrReader> = EventIterator::from(input);
    scan.for_each(|ev| {
        line.push_str("\n");
        write!(line, "{:}", ev).unwrap();
    });

    assert_eq!(line, events, "Error in {input}");
}
