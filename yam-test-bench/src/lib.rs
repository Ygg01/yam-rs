use std::fmt::Write;

use yam_core::tokenizer::{Event, EventIterator, StrReader};

///
/// Assert that in for given input, the parser generates expected set of events
///
/// # Panics
///
///    Function panics if there is a difference between expected events string and one generated
///    from the input.
pub fn assert_eq_event(input: &str, events: &str) {
    let mut line = String::new();
    let scan: EventIterator<StrReader> = EventIterator::from(input);
    for ev in scan {
        line.push('\n');
        write!(line, "{ev:}").unwrap();
        if matches!(ev, Event::ErrorEvent) {
            break;
        }
    }

    assert_eq!(line, events, "Error in {input}");
}

///
/// Assert that in for given input, the parser generates expected set of events
///
/// # Panics
///
///    Function panics if there is a difference between expected events string and one generated
///    from the input.
pub fn assert_eq_event_exact(input: &str, events: &str) {
    let mut line = String::with_capacity(events.len());
    let scan: EventIterator<'_, StrReader, _> = EventIterator::from(input);
    scan.for_each(|ev| {
        line.push('\n');
        write!(line, "{ev:}").unwrap();
    });

    assert_eq!(line, events, "Error in {input}");
}
