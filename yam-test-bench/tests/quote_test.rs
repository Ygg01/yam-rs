use rstest::rstest;
use yam_test_bench::assert_eq_event;
use yam_test_bench::consts::*;

#[rstest]
#[case::single_quote_test1(SQUOTE_STR1_INPUT, SQUOTE_STR_EVENTS)]
#[case::single_quote_test2(SQUOTE_STR2_INPUT, SQUOTE_STR_EVENTS)]
#[case::single_quote_escape_input(SQUOTE_ESCAPE_INPUT, SQUOTE_ESCAPE_EVENTS)]
#[case::single_quote_escape_input_multiline(SQUOTE_ESCAPE2_INPUT, SQUOTE_ESCAPE_EVENTS)]
// Double quote tests
#[case::double_quote(DQUOTE_STR1_INPUT, DQUOTE_STR_EVENTS)]
#[case::double_quote_split(DQUOTE_STR2_INPUT, DQUOTE_STR_EVENTS)]
#[case::double_quote_multiline(DQUOTE_MULTI_INPUT, DQUOTE_MULTI_EVENTS)]
// Double quote multiline
#[case::double_quote_multiline1(DQUOTE_MULTI1_INPUT, DQUOTE_MULTI1_EVENTS)]
#[case::double_quote_multiline2(DQUOTE_MULTI2_INPUT, DQUOTE_MULTI2_EVENTS)]
// Double quote EOF
#[case::double_quote_end_input1(DQUOTE_END_INPUT, DQUOTE_END_EVENTS)]
#[case::double_quote_end_input2(DQUOTE_ERR2_INPUT, DQUOTE_ERR2_EVENTS)]
#[case::double_quote_miss_eof(DQUOTE_MISS_EOF_INPUT, DQUOTE_MISS_EOF_EVENTS)]
#[case::double_quote_indent_err(DQUOTE_INDENT_ERR_INPUT, DQUOTE_INDENT_ERR_EVENTS)]
#[case::double_quote_comment_err(DQUOTE_COMMENT_ERR_INPUT, DQUOTE_COMMENT_ERR_EVENTS)]
// Double quote tabs
#[case::double_quote_tabs1(DQUOTE_LEADING_TAB1_INPUT, DQUOTE_LEADING_TAB_EVENTS)]
#[case::double_quote_tabs2(DQUOTE_LEADING_TAB2_INPUT, DQUOTE_LEADING_TAB_EVENTS)]
#[case::double_quote_tabs3(DQUOTE_LEADING_TAB3_INPUT, DQUOTE_LEADING_TAB2_EVENTS)]
#[case::double_quote_tabs4(DQUOTE_LEADING_TAB4_INPUT, DQUOTE_LEADING_TAB2_EVENTS)]
#[case::double_quote_tabs5(DQUOTE_LEADING_TAB5_INPUT, DQUOTE_LEADING_TAB2_EVENTS)]
// Double quote empty
#[case::double_quote_empty(DQUOTE_EMPTY1_INPUT, DQUOTE_EMPTY1_EVENTS)]
// Double quote example tests
#[case::double_quote_x1_g4rs(X1_G4RS_INPUT, X1_G4RS_EVENTS)]
#[case::double_quote_x2_g4rs(X2_G4RS_INPUT, X1_G4RS_EVENTS)]
#[case::double_quote_x3_g4rs(X3_G4RS_INPUT, X3_G4RS_EVENTS)]
#[case::double_quote_6wpf(X_6WPF_INPUT, X_6WPF_EVENTS)]
#[case::double_quote_np9h(X1_NP9H_INPUT, X1_NP9H_EVENTS)]
// Double quote escapes
#[case::test_escape1(DQUOTE_ESC1_INPUT, DQUOTE_ESC1_EVENTS)]
#[case::test_escape2(DQUOTE_ESC2_INPUT, DQUOTE_ESC2_EVENTS)]
#[case::test_escape3(DQUOTE_STR_ESC1_INPUT, DQUOTE_STR_ESC_EVENTS)]
fn run_quote_tests(#[case] input: &str, #[case] expected_events: &str) {
    assert_eq_event(input, expected_events);
}
