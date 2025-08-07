use yam_test_bench::consts::*;
use yam_test_bench::{assert_eq_event, assert_eq_event_exact};

#[test]
fn doc_empty() {
    assert_eq_event(EMPTY_DOC_ERR_INPUT, EMPTY_DOC_ERR_EVENTS);
    assert_eq_event(DOC_EMPTY_TAG_INPUT, DOC_EMPTY_TAG_EVENTS);
    assert_eq_event(EMPTY_DOC_INPUT, EMPTY_DOC_EVENTS);
}

#[test]
fn doc_err_directive() {
    assert_eq_event_exact(ERR_DIRECTIVE4_INPUT, ERR_DIRECTIVE4_EVENTS);
    assert_eq_event_exact(ERR_DIRECTIVE1_INPUT, ERR_DIRECTIVE1_EVENTS);
    assert_eq_event_exact(ERR_DIRECTIVE2_INPUT, ERR_DIRECTIVE2_EVENTS);
    assert_eq_event_exact(ERR_DIRECTIVE3_INPUT, ERR_DIRECTIVE3_EVENTS);
    assert_eq_event_exact(ERR_MULTIDOC_INPUT, ERR_MULTIDOC_EVENTS);
}

#[test]
fn simple_doc() {
    assert_eq_event_exact(SIMPLE_DOC_INPUT, SIMPLE_DOC_EVENTS);
    assert_eq_event_exact(SIMPLE_DOC2_INPUT, SIMPLE_DOC2_EVENTS);
    assert_eq_event_exact(EMPTY1_INPUT, EMPTY_EVENTS);
    assert_eq_event_exact(EMPTY2_INPUT, EMPTY_EVENTS);
    assert_eq_event_exact(NO_DOC_INPUT, NO_DOC_EVENTS);
}

#[test]
fn doc_footer() {
    assert_eq_event(FOOTER_INPUT, FOOTER_EVENTS);
}

#[test]
fn doc_after_stream() {
    assert_eq_event_exact(POST_DOC_ERR_INPUT, POST_DOC_ERR_EVENTS);
}

#[test]
fn doc_multi() {
    assert_eq_event(X1_6ZKB_INPUT, X1_6ZKB_EVENTS);
    assert_eq_event(MULTI_DOC1_INPUT, MULTI_DOC1_EVENTS);
    assert_eq_event(MULTI_DOC3_INPUT, MULTI_DOC3_EVENTS);
    assert_eq_event(MULTI_DOC4_INPUT, MULTI_DOC4_EVENTS);
}

#[test]
fn doc_err() {
    assert_eq_event_exact(DOC_MAP_ERR_INPUT, DOC_MAP_ERR_EVENTS);
}

#[test]
fn doc_after_err() {
    assert_eq_event_exact(X1_3HFZ_INPUT, X1_3HFZ_EVENTS);
    assert_eq_event_exact(X1_9HCY_INPUT, X1_9HCY_EVENTS);
    assert_eq_event_exact(X1_EB22_INPUT, X1_EB22_EVENTS);
}
