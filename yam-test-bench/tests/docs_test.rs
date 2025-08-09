use rstest::rstest;
use yam_test_bench::consts::*;
use yam_test_bench::{assert_eq_event, assert_eq_event_exact};

#[rstest]
#[case::docs_empty_err(EMPTY_DOC_ERR_INPUT, EMPTY_DOC_ERR_EVENTS)]
#[case::docs_empty_tag(DOC_EMPTY_TAG_INPUT, DOC_EMPTY_TAG_EVENTS)]
#[case::docs_empty_input(EMPTY_DOC_INPUT, EMPTY_DOC_EVENTS)]
#[case::docs_multi_doc1(MULTI_DOC1_INPUT, MULTI_DOC1_EVENTS)]
#[case::docs_multi_doc2(MULTI_DOC3_INPUT, MULTI_DOC3_EVENTS)]
#[case::docs_multi_doc3(MULTI_DOC4_INPUT, MULTI_DOC4_EVENTS)]
#[case::docs_footer(FOOTER_INPUT, FOOTER_EVENTS)]
#[case::docs_6zkb(X1_6ZKB_INPUT, X1_6ZKB_EVENTS)]
fn run_block_tests(#[case] input: &str, #[case] expected_events: &str) {
    assert_eq_event(input, expected_events);
}

#[rstest]
#[case::docs_exact_simple1(SIMPLE_DOC_INPUT, SIMPLE_DOC_EVENTS)]
#[case::docs_exact_simple2(SIMPLE_DOC2_INPUT, SIMPLE_DOC2_EVENTS)]
#[case::docs_exact_empty1(EMPTY1_INPUT, EMPTY_EVENTS)]
#[case::docs_exact_empty2(EMPTY2_INPUT, EMPTY_EVENTS)]
#[case::docs_exact_no_doc(NO_DOC_INPUT, NO_DOC_EVENTS)]
#[case::docs_exact_err_multidoc(ERR_MULTIDOC_INPUT, ERR_MULTIDOC_EVENTS)]
#[case::docs_exact_err_post(POST_DOC_ERR_INPUT, POST_DOC_ERR_EVENTS)]
#[case::docs_exact_err_input(DOC_MAP_ERR_INPUT, DOC_MAP_ERR_EVENTS)]
#[case::docs_exact_3hfz(X1_3HFZ_INPUT, X1_3HFZ_EVENTS)]
#[case::docs_exact_9hcy(X1_9HCY_INPUT, X1_9HCY_EVENTS)]
#[case::docs_exact_eb22(X1_EB22_INPUT, X1_EB22_EVENTS)]
#[case::docs_exact_err_directive1(ERR_DIRECTIVE1_INPUT, ERR_DIRECTIVE1_EVENTS)]
#[case::docs_exact_err_directive2(ERR_DIRECTIVE2_INPUT, ERR_DIRECTIVE2_EVENTS)]
#[case::docs_exact_err_directive3(ERR_DIRECTIVE3_INPUT, ERR_DIRECTIVE3_EVENTS)]
#[case::docs_exact_err_directive4(ERR_DIRECTIVE4_INPUT, ERR_DIRECTIVE4_EVENTS)]
fn simple_doc(#[case] input: &str, #[case] expected_events: &str) {
    assert_eq_event_exact(input, expected_events);
}
