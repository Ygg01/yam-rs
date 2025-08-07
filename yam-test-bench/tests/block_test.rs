use yam_test_bench::consts::*;
use yam_test_bench::{assert_eq_event, assert_eq_event_exact};

#[test]
fn block_seq() {
    assert_eq_event(X1_33X3_INPUT, X1_33X3_EVENTS);
    assert_eq_event(BLOCK1_INPUT, BLOCK_EVENTS);
    assert_eq_event(BLOCK2_INPUT, BLOCK_EVENTS);
    assert_eq_event(SEQ_PLAIN_INPUT, SEQ_PLAIN_EVENTS);
    assert_eq_event(SEQ_PLAIN2_INPUT, SEQ_PLAIN_EVENTS);
}

#[test]
fn block_seq_err() {
    assert_eq_event(X1_P2EQ_INPUT, X1_P2EQ_EVENTS);
    assert_eq_event(SEQ_NO_MINUS_INPUT, SEQ_NO_MINUS_EVENTS);
    assert_eq_event(X_BD7L_INPUT, X_BD7L_EVENTS);
    assert_eq_event(X_9CWY_INPUT, X_9CWY_EVENTS);
    assert_eq_event(BLOCK_ERR_INPUT, BLOCK_ERR_EVENTS);
    assert_eq_event(WRONG_SEQ_INDENT_INPUT, WRONG_SEQ_INDENT_EVENTS);
}

#[test]
fn seq_block_nested() {
    assert_eq_event(X1_3ALJ_INPUT, X_3ALJ_EVENTS);
    assert_eq_event(X2_3ALJ_INPUT, X_3ALJ_EVENTS);
    assert_eq_event(BLOCK_NESTED_SEQ2_INPUT, BLOCK_NESTED_SEQ2_EVENTS);
}

#[test]
fn block_fold() {
    assert_eq_event(FOLD_STR1_INPUT, FOLD_STR1_EVENTS);
    assert_eq_event(FOLD_STR2_INPUT, FOLD_STR2_EVENTS);
    assert_eq_event(FOLD_ERR_INPUT, FOLD_ERR_EVENTS);
}

#[test]
fn block_plain_scalar() {
    assert_eq_event(BLOCK_MULTI_INPUT, BLOCK_MULTI_EVENTS);
    assert_eq_event(BLOCK_PLAIN_INPUT, BLOCK_PLAIN_EVENTS);
    assert_eq_event(BLOCK_PLAIN2_INPUT, BLOCK_PLAIN2_EVENTS);
}

#[test]
fn block_fold_literal() {
    assert_eq_event(X1_X4QW_INPUT, X1_X4QW_EVENTS);
    assert_eq_event(X2_X4QW_INPUT, X2_X4QW_EVENTS);
    assert_eq_event(BLOCK_FOLD_INPUT, BLOCK_FOLD_EVENTS);
    assert_eq_event(SIMPLE_FOLD1_INPUT, SIMPLE_FOLD_EVENTS);
    assert_eq_event(SIMPLE_FOLD2_INPUT, SIMPLE_FOLD_EVENTS);
}

#[test]
fn block_literal() {
    assert_eq_event(LITERAL_ESCAPE_INPUT, LITERAL_ESCAPE_EVENTS);
    assert_eq_event(LITERAL1_INPUT, SIMPLE_FOLDED_EVENTS);
    assert_eq_event(LITERAL2_INPUT, SIMPLE_FOLDED_EVENTS);
    assert_eq_event(BLOCK_QUOTE_INPUT, BLOCK_QUOTE_EVENTS);
    assert_eq_event(LITERAL_CHOMP_INPUT, LITERAL_CHOMP_EVENTS);
    assert_eq_event(LITERAL3_INPUT, LITERAL3_EVENTS);
    assert_eq_event(LIT_STR2_INPUT, LIT_STR2_EVENTS);
    assert_eq_event(MULTILINE_PLAIN_INPUT, MULTILINE_PLAIN_EVENTS);
}

#[test]
fn block_literal_indents() {
    assert_eq_event(X1_Y79Y_000_INPUT, X1_Y79Y_000_EVENTS);
    assert_eq_event(X2_Y79Y_000_INPUT, X2_Y79Y_000_EVENTS);
    assert_eq_event(X3_Y79Y_000_INPUT, X3_Y79Y_000_EVENTS);
    assert_eq_event(X4_Y79Y_000_INPUT, X4_Y79Y_000_EVENTS);
}

#[test]
fn block_literal_err() {
    assert_eq_event(LITERAL_ERR_INPUT, SIMPLE_FOLDED_ERR_EVENTS);
    assert_eq_event(LITERAL_ERR2_INPUT, SIMPLE_FOLDED_ERR_EVENTS);
}

#[test]
fn block_indent_lit_fold() {
    assert_eq_event(X2_7T8X_INPUT, X2_7T8X_EVENTS);
    assert_eq_event(X1_7T8X_INPUT, X1_7T8X_EVENTS);
    assert_eq_event(X1_6VJK_INPUT, X1_6VJK_EVENTS);
    assert_eq_event(X2_6VJK_INPUT, X2_6VJK_EVENTS);
    assert_eq_event(X1_JEF9_INPUT, X1_JEF9_EVENTS);
    assert_eq_event(X1_F6MC_INPUT, X1_F6MC_EVENTS);
    assert_eq_event(X2_F6MC_INPUT, X2_F6MC_EVENTS);
}

#[test]
fn block_plain_multiline() {
    assert_eq_event(PLAIN_MULTI_INPUT, PLAIN_MULTI_EVENTS);
    assert_eq_event(X_8XDJ_INPUT, X_8XDJ_EVENTS);
}

#[test]
fn block_map() {
    assert_eq_event(X1_1_SYW4_INPUT, X1_SYW4_EVENTS);
    assert_eq_event(X1_2_SYW4_INPUT, X1_SYW4_EVENTS);

    assert_eq_event(MAP_SIMPLE_INPUT, MAP_SIMPLE_EVENTS);
    assert_eq_event(MAP_SIMPLE2_INPUT, MAP_SIMPLE_EVENTS);
}

#[test]
fn block_quote_map() {
    assert_eq_event(DQUOTE_MAP_INPUT, DQUOTE_MAP_EVENTS);
    assert_eq_event(DQUOTE_MUL_INPUT, DQUOTE_MUL_EVENTS);
}

#[test]
fn block_empty_map() {
    assert_eq_event_exact(X1_NKF9_INPUT, X1_NKF9_EVENTS);
    assert_eq_event(X1_6KGN_INPUT, X1_6KGN_EVENTS);
    assert_eq_event(NESTED_EMPTY_INPUT, NESTED_EMPTY_EVENTS);

    assert_eq_event(EMPTY_MAP_INPUT, EMPTY_MAP_EVENTS);
    assert_eq_event(MULTI_EMPTY_INPUT, MULTI_EMPTY_EVENTS);

    assert_eq_event(EMPTY_KEY_MAP2_INPUT, EMPTY_KEY_MAP2_EVENTS);
    assert_eq_event(EMPTY_KEY_MAP2_1_INPUT, EMPTY_KEY_MAP2_EVENTS);
    assert_eq_event(MIX_EMPTY_MAP_INPUT, MIX_EMPTY_MAP_EVENTS);
    assert_eq_event(MAP2_INPUT, MAP2_EVENTS);
}

#[test]
fn block_multiline_comment() {
    assert_eq_event(MULTILINE_COMMENT1_INPUT, MULTILINE_COMMENT1_EVENTS);
    assert_eq_event(MULTILINE_COMMENT1_2_INPUT, MULTILINE_COMMENT1_EVENTS);
    assert_eq_event(MULTILINE_COMMENT2_INPUT, MULTILINE_COMMENT2_EVENTS);
    assert_eq_event(MULTILINE_COMMENT3_INPUT, MULTILINE_COMMENT3_EVENTS);
}

#[test]
fn block_exp_map() {
    assert_eq_event(X1_V9D5_INPUT, X1_V9D5_EVENTS);
    assert_eq_event(X1_2XXW_INPUT, X1_2XXW_EVENTS);
    assert_eq_event(X1_A2M4_INPUT, X1_A2M4_EVENTS);
    assert_eq_event(X_7W2P_INPUT, X_7W2P_EVENTS);
    assert_eq_event(EXP_MAP_FOLD_INPUT, EXP_MAP_FOLD_EVENTS);
    assert_eq_event(X_5WE3_INPUT, X_5WE3_EVENTS);
    assert_eq_event(EXP_MAP_INPUT, EXP_MAP_EVENTS);
    assert_eq_event(EXP_BLOCK_MAP_MIX_INPUT, EXP_BLOCK_MAP_MIX_EVENTS);
    assert_eq_event(EXP_MAP_COMP_INPUT, EXP_MAP_COMP_EVENTS);
}

#[test]
fn block_empty_node_exp_map() {
    assert_eq_event(EXP_MAP_EMPTY_INPUT, EXP_MAP_EMPTY_INPUT_EVENTS);
    assert_eq_event(EXP_MAP_FAKE_EMPTY_INPUT, EXP_MAP_FAKE_EMPTY_EVENTS);
}

#[test]
fn block_empty_node_map() {
    assert_eq_event(EMPTY_KEY_MAP_INPUT, EMPTY_KEY_MAP_EVENTS);
}

#[test]
fn block_exp_map_err() {
    assert_eq_event(EXP_BLOCK_MAP_ERR1, EXP_BLOCK_MAP_ERR1_EVENTS);
    assert_eq_event(EXP_BLOCK_MAP_ERR2, EXP_BLOCK_MAP_ERR2_EVENTS);
}

#[test]
fn block_map_inline_err() {
    assert_eq_event(INLINE_ERR_INPUT, INLINE_ERR_EVENTS);
}

#[test]
fn block_map_err() {
    assert_eq_event(ERR_MULTILINE_KEY_INPUT, ERR_MULTILINE_KEY_EVENTS);
    assert_eq_event(ERR_TRAIL_INPUT, ERR_TRAIL_EVENTS);
    assert_eq_event(ERR_INVALID_KEY1_INPUT, ERR_INVALID_KEY1_EVENTS);
    assert_eq_event(ERR_INVALID_KEY2_INPUT, ERR_INVALID_KEY2_EVENTS);
    assert_eq_event(ERR_INVALID_KEY3_INPUT, ERR_INVALID_KEY3_EVENTS);
}

#[test]
fn block_map_complex() {
    assert_eq_event(COMPLEX_NESTED_INPUT, COMPLEX_NESTED_EVENTS);
    assert_eq_event(NESTED_INPUT, NESTED_EVENTS);
    assert_eq_event(COMPLEX_KEYS_INPUT, COMPLEX_KEYS_EVENTS);
    assert_eq_event(X1_9C9N_INPUT, X1_9C9N_EVENTS);
    assert_eq_event(MAP_AND_COMMENT_INPUT, MAP_AND_COMMENT_EVENTS);
}

#[test]
fn block_flow_mix() {
    assert_eq_event(X1_4AW9_INPUT, X1_4AW9_EVENTS);
    assert_eq_event(X1_87E4_INPUT, X_87E4_EVENTS);
    assert_eq_event(X1_6HB6_INPUT, X1_6HB6_EVENTS);
    assert_eq_event(X_7ZZ5_INPUT, X_7ZZ5_EVENTS);
    assert_eq_event(X2_87E4_INPUT, X_87E4_EVENTS);
    assert_eq_event(X_8KB6_INPUT, X_8KB6_EVENTS);
}

#[test]
fn block_map_scalar_and_ws() {
    assert_eq_event(MAPS_WITH_QUOTES_INPUT, MAPS_WITH_QUOTES_EVENTS);
}

#[test]
fn block_nested_maps() {
    assert_eq_event(X1_Q9WF_INPUT, X1_Q9WF_EVENTS);
    assert_eq_event(NESTED_MAPS_INPUT, NESTED_MAPS_EVENTS);
}

#[test]
fn block_map_anchor_alias() {
    assert_eq_event(ALIAS_N_MAPS_INPUT, ALIAS_N_MAPS_EVENTS);
    assert_eq_event(ALIAS_N_MAPS2_INPUT, ALIAS_N_MAPS2_EVENTS);
    assert_eq_event(ALIAS_N_COMP_MAP_INPUT, ALIAS_N_COMP_MAP_EVENTS);
}

#[test]
fn block_seq_anchor_alias_err() {
    assert_eq_event(X1_SR86_INPUT, X1_SR86_EVENTS);
}

#[test]
fn block_exp_map_alias() {
    assert_eq_event(X3_PW8X_INPUT, X3_PW8X_EVENTS);
    assert_eq_event(X2_PW8X_INPUT, X2_PW8X_EVENTS);
    assert_eq_event(X1_PW8X_INPUT, X1_PW8X_EVENTS);
}

#[test]
fn block_seq_anchor_alias() {
    assert_eq_event(X1_HMQ5_INPUT, X1_HMQ5_EVENTS);

    assert_eq_event(X1_G9HC_INPUT, X1_G9HC_EVENTS);
    assert_eq_event(X2_1_G9HC_INPUT, X2_G9HC_EVENTS);
    assert_eq_event(X2_2_G9HC_INPUT, X2_G9HC_EVENTS);

    assert_eq_event(ALIAS_N_SEQ1_INPUT, ALIAS_N_SEQ1_EVENTS);
    assert_eq_event(ALIAS_N_SEQ2_INPUT, ALIAS_N_SEQ2_EVENTS);
    assert_eq_event(ALIAS_N_SEQ3_INPUT, ALIAS_N_SEQ3_EVENTS);
}

#[test]
fn block_col_tags() {
    assert_eq_event(X3_57H4_INPUT, X3_57H4_EVENTS);
    assert_eq_event(X2_57H4_INPUT, X2_57H4_EVENTS);
    assert_eq_event(X1_57H4_INPUT, X1_57H4_EVENTS);
    assert_eq_event(TAG_DEF_INPUT, TAG_DEF_EVENTS);
    assert_eq_event(EXP_TAG_INPUT, EXP_TAG_EVENTS);
}

#[test]
fn block_anchor() {
    assert_eq_event(X1_735Y_INPUT, X1_735Y_EVENTS);
    assert_eq_event(ANCHOR_COLON_INPUT, ANCHOR_COLON_EVENTS);
    assert_eq_event(ANCHOR_MULTI_2_INPUT, ANCHOR_MULTI_2_EVENTS);
    assert_eq_event(ANCHOR_MULTI_INPUT, ANCHOR_MULTI_EVENTS);
    assert_eq_event(ANCHOR_ERR_INPUT, ANCHOR_ERR_EVENTS);
}

#[test]
fn block_mix_seq() {
    assert_eq_event(MIX_BLOCK_INPUT, MIX_BLOCK_EVENTS);
    assert_eq_event(MIX2_BLOCK_INPUT, MIX2_BLOCK_EVENTS);
}

#[test]
fn block_tag() {
    assert_eq_event(TAG1_1_INPUT, TAG1_EVENTS);
    assert_eq_event(TAG1_2_INPUT, TAG1_EVENTS);
    assert_eq_event(COMPLEX_TAG2_INPUT, COMPLEX_TAG2_EVENTS);
    assert_eq_event(X_74H7_INPUT, X_74H7_EVENTS);
}

#[test]
fn block_multi_line() {
    assert_eq_event(MULTI_LINE_INPUT, MULTI_LINE_EVENTS);
    assert_eq_event(MULTI_LINE_SEQ_INPUT, MULTI_LINE_SEQ_EVENTS);
    assert_eq_event(X_BF9H_INPUT, X_BF9H_EVENTS);
    assert_eq_event(X_BS4K_INPUT, X_BS4K_EVENTS);
}

#[test]
fn block_seq_and_map() {
    assert_eq_event(X1_S7BG_INPUT, X1_S7BG_EVENTS);
    assert_eq_event(SEQ_SAME_LINE_INPUT, SEQ_SAME_LINE_EVENTS);
}

#[test]
fn block_tag_short() {
    assert_eq_event(X1_U99R_INPUT, X1_U99R_EVENTS);
    assert_eq_event(X2_U99R_INPUT, X2_U99R_EVENTS);

    assert_eq_event(X1_QLJ7_INPUT, X1_QLJ7_EVENTS);
    assert_eq_event(X1_TAG_SHORT_INPUT, X1_TAG_SHORT_EVENTS);
    assert_eq_event(TAG_SHORT_INPUT, TAG_SHORT_EVENTS);
}

#[test]
fn block_tag_anchor() {
    assert_eq_event(X5_9KAX_INPUT, X5_9KAX_EVENTS);
    assert_eq_event(X4_9KAX_INPUT, X4_9KAX_EVENTS);
    assert_eq_event(X1_9KAX_INPUT, X1_9KAX_EVENTS);
    assert_eq_event(X2_9KAX_INPUT, X1_9KAX_EVENTS);
    assert_eq_event(X3_9KAX_INPUT, X3_9KAX_EVENTS);

    assert_eq_event(X1_6JWB_INPUT, X1_6JWB_EVENTS);
}

#[test]
fn block_map_tab() {
    assert_eq_event(X1_DK95_INPUT, X1_DK95_EVENTS);
    assert_eq_event(X2_DK95_INPUT, X2_DK95_EVENTS);
    assert_eq_event(X3_DK95_INPUT, X3_DK95_EVENTS);
}

#[test]
fn block_map_err_indent() {
    assert_eq_event(X1_U44R_INPUT, X1_U44R_EVENTS);
    assert_eq_event(X1_EW3V_INPUT, X1_EW3V_EVENTS);
    assert_eq_event(X1_DMG6_INPUT, X1_DMG6_EVENTS);
    assert_eq_event(X1_7LBH_INPUT, X1_7LBH_EVENTS);
}

#[test]
fn block_seq_empty() {
    assert_eq_event(SEQ_EMPTY1_INPUT, SEQ_EMPTY1_EVENTS);
    assert_eq_event(SEQ_EMPTY2_INPUT, SEQ_EMPTY2_EVENTS);
}

#[test]
fn block_tab_indents() {
    assert_eq_event(X1_DC7X_INPUT, X1_DC7X_EVENTS);

    assert_eq_event(X1_Y79Y_001_INPUT, X1_Y79Y_001_EVENTS);

    assert_eq_event(X1_Y79Y_004_INPUT, X1_Y79Y_004_EVENTS);
    assert_eq_event(X2_Y79Y_004_INPUT, X2_Y79Y_004_EVENTS);
    assert_eq_event(X3_Y79Y_004_INPUT, X2_Y79Y_004_EVENTS);

    assert_eq_event(X1_Y79Y_006_INPUT, X1_Y79Y_006_EVENTS);
    assert_eq_event(X2_Y79Y_006_INPUT, X2_Y79Y_006_EVENTS);
    assert_eq_event(X3_Y79Y_006_INPUT, X3_Y79Y_006_EVENTS);

    assert_eq_event(X1_Y79Y_007_INPUT, X1_Y79Y_007_EVENTS);

    assert_eq_event(X1_Y79Y_009_INPUT, X1_Y79Y_009_EVENTS);
    assert_eq_event(X2_Y79Y_009_INPUT, X2_Y79Y_009_EVENTS);
    assert_eq_event(X3_Y79Y_009_INPUT, X3_Y79Y_009_EVENTS);
}

#[test]
fn block_tags_empty() {
    assert_eq_event(X1_UKK6_02_INPUT, X1_UKK6_02_EVENTS);
    assert_eq_event(X4_FH7J_INPUT, X4_FH7J_EVENTS);
    assert_eq_event(X3_FH7J_INPUT, X3_FH7J_EVENTS);
    assert_eq_event(X1_FH7J_INPUT, X1_FH7J_EVENTS);
    assert_eq_event(X2_FH7J_INPUT, X2_FH7J_EVENTS);
}

#[test]
fn block_chomp() {
    assert_eq_event(X1_MJS9_INPUT, X1_MJS9_EVENTS);
    assert_eq_event(X1_K858_INPUT, X1_K858_EVENTS);
}

#[test]
fn block_complex_exp_mix() {
    assert_eq_event(X1_M5DY_INPUT, X1_M5DY_EVENTS);
    assert_eq_event(X2_M5DY_INPUT, X2_M5DY_EVENTS);

    assert_eq_event(X2_KK5P_INPUT, X2_KK5P_EVENTS);
    assert_eq_event(X1_KK5P_INPUT, X1_KK5P_EVENTS);
}

#[test]
fn block_comment() {
    assert_eq_event_exact(X1_RZP5_INPUT, X1_RZP5_EVENTS)
}
