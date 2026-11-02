extern crate yam_core;

use yam_core::parsing::{Event, EventReceiver, Parser};

struct EventSink<'input> {
    events: Vec<Event<'input>>,
}

impl<'input> EventReceiver<'input> for EventSink<'input> {
    fn on_event(&mut self, ev: Event<'input>) {
        self.events.push(ev);
    }
}

fn main() {
    let yaml_str = "{a: b, c: d}";
    let mut sink = EventSink {
        events: Vec::with_capacity(10),
    };
    let mut parser = Parser::new_from_str(yaml_str);

    if parser.load(&mut sink, true).is_err() {
        return;
    };

    for x in sink.events {
        println!("{:?}", x);
    }
}
