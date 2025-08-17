extern crate libtest_mimic;

use std::error::Error;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::{fs, io};

use libtest_mimic::{Arguments, Failed, Trial};
use std::fmt::Write;
use yam_common::Event;
use yam_core::tokenizer::EventIterator;

const TEST_SIZE: usize = 440;

#[derive(Default)]
struct TestData {
    desc: String,
    input_yaml: PathBuf,
    input_json: Option<PathBuf>,
    is_error: bool,
    test_event: PathBuf,
    output_yaml: Option<PathBuf>,
    emit_yaml: Option<PathBuf>,
}

fn perform_test(data: TestData, is_strict: bool) -> Result<(), Failed> {
    let input_yaml = fs::read_to_string(data.input_yaml)?;
    let mut actual_event = String::with_capacity(input_yaml.len());
    let ev_iterator = EventIterator::from(&*input_yaml);
    actual_event.push_str("+STR\r\n");
    let mut is_error = false;
    for ev in ev_iterator {
        if matches!(ev, Event::Directive { .. }) {
            continue;
        }
        if ev == Event::ErrorEvent {
            is_error = true;
            break;
        }
        write!(actual_event, "{:}", ev)?;
        actual_event.push_str("\r\n");
    }
    if !is_error {
        actual_event.push_str("-STR\r\n");
    }

    if is_strict || !is_error {
        let expected_event = adjusted_test_event(data.test_event)?;
        assert_eq!(actual_event, expected_event);
    } else {
        assert_eq!(is_error, data.is_error);
    }

    Ok(())
}

fn adjusted_test_event(path: PathBuf) -> io::Result<String> {
    let transform_events = fs::read_to_string(&path)?
        .replace("+DOC ---", "+DOC")
        .replace("-DOC ...", "-DOC")
        .replace("+MAP {}", "+MAP")
        .replace("+SEQ []", "+SEQ");
    Ok(transform_events)
}

fn collect_test_suite(
    path: &Path,
    ignore_list: Vec<&str>,
    tests: &mut Vec<Trial>,
    is_strict: bool,
) -> Result<(), Box<dyn Error>> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let test_dir_path = entry.path();
        let dir_name = entry
            .file_name()
            .into_string()
            .expect("non-UTF8 string in path");
        if file_type.is_dir() && !ignore_list.contains(&&*dir_name) {
            collect_test(dir_name, &test_dir_path, &ignore_list, tests, is_strict)?;
        }
    }
    Ok(())
}

fn collect_test(
    dir_name: String,
    test_dir_path: &PathBuf,
    ignore_list: &Vec<&str>,
    tests: &mut Vec<Trial>,
    is_strict: bool,
) -> Result<(), Box<dyn Error>> {
    let mut test_data = TestData::default();
    let mut is_dir = false;
    for entry in fs::read_dir(test_dir_path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let filename = entry
            .file_name()
            .into_string()
            .expect("non-UTF8 string in path");
        if file_type.is_dir() && !ignore_list.contains(&filename.deref()) {
            let sub_dir = entry
                .file_name()
                .into_string()
                .expect("non-UTF8 string in path");
            let dir_name = format!("{dir_name}/{sub_dir}");
            let subdir_path = entry.path();
            collect_test(dir_name, &subdir_path, ignore_list, tests, is_strict)?;
            is_dir = true;
        } else {
            match &*filename {
                "===" => {
                    if let Ok(desc) = fs::read_to_string(entry.path()) {
                        test_data.desc = String::from(desc.trim());
                    }
                }
                "in.yaml" => test_data.input_yaml = entry.path(),
                "in.json" => test_data.input_json = Some(entry.path()),
                "error" => test_data.is_error = true,
                "test.event" => test_data.test_event = entry.path(),
                "out.yaml" => test_data.output_yaml = Some(entry.path()),
                "emit.yaml" => test_data.emit_yaml = Some(entry.path()),
                _ => {}
            };
        }
    }
    if !is_dir {
        let test = Trial::test(format!("{} ({})", dir_name, &test_data.desc), move || {
            perform_test(test_data, is_strict)
        });
        tests.push(test);
    }

    Ok(())
}

fn collect_tests(
    path: &Path,
    filter_list: Vec<&str>,
    is_strict: bool,
) -> Result<Vec<Trial>, Box<dyn Error>> {
    let mut tests = Vec::with_capacity(TEST_SIZE);
    collect_test_suite(path, filter_list, &mut tests, is_strict)?;
    Ok(tests)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();
    let filter_list = vec![".git", "name", "tags"];

    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("yaml-test-suite");

    let tests = collect_tests(&path, filter_list, false)?;

    libtest_mimic::run(&args, tests).exit();
    // Ok(())
}
