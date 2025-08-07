use yam_test_bench::assert_eq_event;
use yam_test_bench::consts::*;

#[test]
fn quote_single() {
    assert_eq_event(SQUOTE_STR1_INPUT, SQUOTE_STR_EVENTS);
    assert_eq_event(SQUOTE_STR2_INPUT, SQUOTE_STR_EVENTS);
    assert_eq_event(SQUOTE_ESCAPE_INPUT, SQUOTE_ESCAPE_EVENTS);
    assert_eq_event(SQUOTE_ESCAPE2_INPUT, SQUOTE_ESCAPE_EVENTS);
}

#[test]
fn dquote_solo() {
    assert_eq_event(DQUOTE_STR1_INPUT, DQUOTE_STR_EVENTS);
    assert_eq_event(DQUOTE_STR2_INPUT, DQUOTE_STR_EVENTS);
    assert_eq_event(DQUOTE_MULTI_INPUT, DQUOTE_MULTI_EVENTS);
}

#[test]
fn dquote_multiline() {
    assert_eq_event(DQUOTE_MULTI1_INPUT, DQUOTE_MULTI1_EVENTS);
    assert_eq_event(DQUOTE_MULTI2_INPUT, DQUOTE_MULTI2_EVENTS);
    assert_eq_event(X_6WPF_INPUT, X_6WPF_EVENTS);
}

#[test]
fn dquote_err() {
    assert_eq_event(DQUOTE_END_INPUT, DQUOTE_END_EVENTS);
    assert_eq_event(DQUOTE_ERR2_INPUT, DQUOTE_ERR2_EVENTS);
    assert_eq_event(DQUOTE_MISS_EOF_INPUT, DQUOTE_MISS_EOF_EVENTS);
    assert_eq_event(DQUOTE_INDENT_ERR_INPUT, DQUOTE_INDENT_ERR_EVENTS);
    assert_eq_event(DQUOTE_COMMENT_ERR_INPUT, DQUOTE_COMMENT_ERR_EVENTS);
}

#[test]
fn dquote_trailing() {
    assert_eq_event(DQUOTE_LEADING_TAB1_INPUT, DQUOTE_LEADING_TAB_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB2_INPUT, DQUOTE_LEADING_TAB_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB3_INPUT, DQUOTE_LEADING_TAB2_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB4_INPUT, DQUOTE_LEADING_TAB2_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB5_INPUT, DQUOTE_LEADING_TAB2_EVENTS);
}

#[test]
fn dquote_empty() {
    assert_eq_event(DQUOTE_EMPTY1_INPUT, DQUOTE_EMPTY1_EVENTS);
}

#[test]
fn dquote_escape_unicode() {
    assert_eq_event(X3_G4RS_INPUT, X3_G4RS_EVENTS);
    assert_eq_event(X1_G4RS_INPUT, X1_G4RS_EVENTS);
    assert_eq_event(X2_G4RS_INPUT, X1_G4RS_EVENTS);
}

#[test]
fn dquote_escape() {
    assert_eq_event(X1_NP9H_INPUT, X1_NP9H_EVENTS);
    assert_eq_event(DQUOTE_ESC1_INPUT, DQUOTE_ESC1_EVENTS);
    assert_eq_event(DQUOTE_ESC2_INPUT, DQUOTE_ESC2_EVENTS);
    assert_eq_event(DQUOTE_STR_ESC1_INPUT, DQUOTE_STR_ESC_EVENTS);
}
