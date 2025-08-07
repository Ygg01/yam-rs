use yam_test_bench::assert_eq_event;
use yam_test_bench::consts::*;

#[test]
fn flow_scalars() {
    assert_eq_event(NULL_YAML_INPUT, NULL_YAML_EVENTS);
    assert_eq_event(NULL_YAML2_INPUT, NULL_YAML_EVENTS);
    assert_eq_event(MULTI_WORD_INPUT, MULTI_WORD_EVENTS);
    assert_eq_event(MULTILINE_INPUT, MULTILINE_EVENTS);
}

#[test]
fn flow_seq() {
    assert_eq_event(SEQ_FLOW_INPUT, SEQ_FLOW_EVENTS);
    assert_eq_event(SEQ_FLOW2_INPUT, SEQ_FLOW_EVENTS);
}

#[test]
fn flow_implicit_map_in_seq() {
    assert_eq_event(NEST_COL1_INPUT, NESTED_COL_EVENTS);
    assert_eq_event(NEST_COL2_INPUT, NESTED_COL_EVENTS);
}

#[test]
fn flow_complex_map() {
    assert_eq_event(COMPLEX_MAP_INPUT, COMPLEX_MAP_EVENTS);
    assert_eq_event(X1_9MMW_INPUT, X1_9MMW_EVENTS);
    assert_eq_event(X2_9MMW_INPUT, X2_9MMW_EVENTS);
}

#[test]
fn flow_map() {
    assert_eq_event(X1_1_ZXT5_INPUT, X1_ZXT5_EVENTS);
    assert_eq_event(X1_2_ZXT5_INPUT, X1_ZXT5_EVENTS);
    assert_eq_event(X1_WZ62_INPUT, X1_WZ62_EVENTS);
    assert_eq_event(MAP_XY_INPUT, MAP_XY_EVENTS);
    assert_eq_event(MAP_X_Y2_INPUT, MAP_X_Y_EVENTS);
    assert_eq_event(MAP_X_Y_INPUT, MAP_X_Y_EVENTS);
    assert_eq_event(MAP_X_Y3_INPUT, MAP_X_Y_EVENTS);
}

#[test]
fn flow_map_quoted() {
    assert_eq_event(FLOW_QUOTED2_INPUT, FLOW_QUOTED2_EVENTS);
    assert_eq_event(FLOW_QUOTED1_INPUT, FLOW_QUOTED1_EVENTS);
    assert_eq_event(X_C2DT_INPUT, X_C2DT_EVENTS);
}

#[test]
fn flow_empty_nodes() {
    assert_eq_event(EMPTY_MAP1_INPUT, EMPTY_FLOW_MAP_EVENTS);
    assert_eq_event(EMPTY_MAP2_INPUT, EMPTY_FLOW_MAP_EVENTS);
    assert_eq_event(TWO_EMPTY_INPUT, TWO_EMPTY_EVENTS);
    assert_eq_event(EMPTY_NODES_INPUT, EMPTY_NODES_EVENTS);
}

#[test]
fn flow_err_plain_scalar() {
    assert_eq_event(ERR_PLAIN_SCALAR_INPUT, ERR_PLAIN_SCALAR_EVENTS);
}

#[test]
fn flow_seq_err() {
    assert_eq_event(X1_N782_INPUT, X1_N782_EVENTS);
    assert_eq_event(X2_N782_INPUT, X2_N782_EVENTS);

    assert_eq_event(X2_CVW2_INPUT, X2_CVW2_EVENTS);
    assert_eq_event(X1_CVW2_INPUT, X1_CVW2_EVENTS);
    assert_eq_event(X_CML9_INPUT, X_CML9_EVENTS);
    assert_eq_event(FLOW_ERR2_INPUT, FLOW_ERR2_EVENTS);
    assert_eq_event(FLOW_ERR1_INPUT, FLOW_ERR1_EVENTS);
    assert_eq_event(SEQ_ERR_INPUT, SEQ_ERR_EVENTS);
    assert_eq_event(X_9JBA_INPUT, X_9JBA_EVENTS);
    assert_eq_event(X_9MAG_INPUT, X_9MAG_EVENTS);
}

#[test]
fn flow_seq_as_key() {
    assert_eq_event(SEQ_KEY1_INPUT, SEQ_KEY1_EVENTS);
    assert_eq_event(SEQ_KEY2_INPUT, SEQ_KEY2_EVENTS);
    assert_eq_event(SEQ_KEY3_INPUT, SEQ_KEY3_EVENTS);
    assert_eq_event(SEQ_KEY4_INPUT, SEQ_KEY4_EVENTS);
}

#[test]
fn flow_seq_edge() {
    assert_eq_event(X1_8UDB_INPUT, X1_8UDB_EVENTS);
    assert_eq_event(X2_8UDB_INPUT, X2_8UDB_EVENTS);
    assert_eq_event(SEQ_EDGE_INPUT, SEQ_EDGE_EVENTS);
}

#[test]
fn flow_map_edge() {
    assert_eq_event(X1_T833_INPUT, X1_T833_EVENTS);
    assert_eq_event(X_CT4Q_INPUT, X_CT4Q_EVENTS);
    assert_eq_event(X1_DK4H_INPUT, X1_DK4H_EVENTS);
    assert_eq_event(X2_DK4H_INPUT, X2_DK4H_EVENTS);
    assert_eq_event(X_DFF7_INPUT, X_DFF7_EVENTS);
    assert_eq_event(MAP_EDGE1_INPUT, MAP_EDGE1_EVENTS);
    assert_eq_event(MAP_EDGE2_INPUT, MAP_EDGE2_EVENTS);
    assert_eq_event(MAP_ERR_INPUT, MAP_ERR_EVENTS);
}

#[test]
fn flow_custom_tag() {
    assert_eq_event(X1_EHF6_INPUT, X1_EHF6_EVENTS);
    assert_eq_event(FLOW_TAG_INPUT, FLOW_TAG_EVENTS);
}

#[test]
fn flow_anchor() {
    assert_eq_event(X1_X38W_INPUT, X1_X38W_EVENTS);

    assert_eq_event(X4_CN3R_INPUT, X4_CN3R_EVENTS);
    assert_eq_event(X3_CN3R_INPUT, X3_CN3R_EVENTS);
    assert_eq_event(X2_CN3R_INPUT, X2_CN3R_EVENTS);
    assert_eq_event(X1_CN3R_INPUT, X1_CN3R_EVENTS);
    assert_eq_event(FLOW_ALIAS_INPUT, FLOW_ALIAS_EVENTS);
}

#[test]
fn flow_in_seq_indents() {
    assert_eq_event(X1_Y79Y_003_INPUT, X1_Y79Y_003_EVENTS);
}

#[test]
fn flow_mix() {
    assert_eq_event(X1_5T43_INPUT, X1_5T43_EVENTS);
}

#[test]
fn flow_exp_map() {
    assert_eq_event(X1_FRK4_INPUT, X1_FRK4_EVENTS);
}
