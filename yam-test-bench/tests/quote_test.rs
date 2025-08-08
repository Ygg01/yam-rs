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
#[case::double_quote_6WPF_snippet(X_6WPF_INPUT, X_6WPF_EVENTS)]
// Double quote EOF
// #[case::test_z01(DQUOTE_END_INPUT, DQUOTE_END_EVENTS)]
// #[case::test_z01(DQUOTE_ERR2_INPUT, DQUOTE_ERR2_EVENTS)]
// #[case::test_z01(DQUOTE_MISS_EOF_INPUT, DQUOTE_MISS_EOF_EVENTS)]
// #[case::test_z01(DQUOTE_INDENT_ERR_INPUT, DQUOTE_INDENT_ERR_EVENTS)]
// #[case::test_z01(DQUOTE_COMMENT_ERR_INPUT, DQUOTE_COMMENT_ERR_EVENTS)]
//
// #[case::double_quote_tabs(DQUOTE_LEADING_TAB1_INPUT, DQUOTE_LEADING_TAB_EVENTS)]
// #[case::double_quote_tabs(DQUOTE_LEADING_TAB2_INPUT, DQUOTE_LEADING_TAB_EVENTS)]
// #[case::double_quote_tabs(DQUOTE_LEADING_TAB3_INPUT, DQUOTE_LEADING_TAB2_EVENTS)]
// #[case::double_quote_tabs(DQUOTE_LEADING_TAB4_INPUT, DQUOTE_LEADING_TAB2_EVENTS)]
// #[case::double_quote_tabs(DQUOTE_LEADING_TAB5_INPUT, DQUOTE_LEADING_TAB2_EVENTS)]
//
// #[case(DQUOTE_EMPTY1_INPUT, DQUOTE_EMPTY1_EVENTS)]
//
// #[case::double_quote_X1_G4RS(X1_G4RS_INPUT, X1_G4RS_EVENTS)]
// #[case::double_quote_X1_G4RS(X2_G4RS_INPUT, X1_G4RS_EVENTS)]
// #[case::double_quote_X1_G4RS(X3_G4RS_INPUT, X3_G4RS_EVENTS)]
//
// #[case::test_war(X1_NP9H_INPUT, X1_NP9H_EVENTS)]
// #[case::test_war(DQUOTE_ESC1_INPUT, DQUOTE_ESC1_EVENTS)]
// #[case::test_war(DQUOTE_ESC2_INPUT, DQUOTE_ESC2_EVENTS)]
// #[case::test_war(DQUOTE_STR_ESC1_INPUT, DQUOTE_STR_ESC_EVENTS)]

fn run_tests(#[case] input: &str, #[case] expected_events: &str) {
    assert_eq_event(input, expected_events);
}
