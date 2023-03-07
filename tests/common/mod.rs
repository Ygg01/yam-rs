use steel_yaml::tokenizer::EventIterator;

extern crate steel_yaml;

pub fn assert_eq_event(input_yaml: &str, expect: &str) {
    let mut event = String::new();
    let scan = EventIterator::new_from_string(input_yaml);
    scan.for_each(|x| event.push_str(x.as_ref()));

    assert_eq!(expect, event, "Error in {input_yaml}");
}
