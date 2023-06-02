extern crate libtest_mimic;

use std::error::Error;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use libtest_mimic::{Arguments, Failed, Trial};
use std::fmt::Write;
use steel_yaml::tokenizer::{Event, EventIterator, StrReader};

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

fn perform_test(data: TestData) -> Result<(), Failed> {
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
    // TODO Input json/output yaml/emit yaml

    let expected_event = fs::read_to_string(data.test_event)?;
    assert_eq!(actual_event, expected_event);

    Ok(())
}

fn collect_test_suite(
    path: &Path,
    ignore_list: Vec<&str>,
    tests: &mut Vec<Trial>,
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
            collect_test(dir_name, &test_dir_path, &ignore_list, tests)?;
        }
    }
    Ok(())
}

fn collect_test(
    dir_name: String,
    test_dir_path: &PathBuf,
    ignore_list: &Vec<&str>,
    tests: &mut Vec<Trial>,
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
            collect_test(dir_name, &subdir_path, ignore_list, tests)?;
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
        let test = Trial::test(format!("{} ({})", dir_name, &test_data.desc), || {
            perform_test(test_data)
        });
        tests.push(test);
    }

    Ok(())
}

fn collect_tests(path: &Path, filter_list: Vec<&str>) -> Result<Vec<Trial>, Box<dyn Error>> {
    let mut tests = Vec::with_capacity(TEST_SIZE);
    collect_test_suite(path, filter_list, &mut tests)?;
    Ok(tests)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Arguments::from_args();
    // args.filter = Some(String::from("2EBW"));
    let filter_list = vec![".git", "name", "tags"];

    let tests = collect_tests(
        Path::new(r#"C:\projects\steel_yaml\tests\yaml-test-suite"#),
        filter_list,
    )?;

    libtest_mimic::run(&args, tests).exit();
}
