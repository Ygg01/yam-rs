use yam_test_bench::assert_eq_event;

const DQUOTE_STR_ESC1_INPUT: &str = r#"
 "double quote (\")""#;

const DQUOTE_STR_ESC_EVENTS: &str = r#"
+DOC
=VAL "double quote (")
-DOC"#;

const DQUOTE_ESC1_INPUT: &str = r#"
 "a\/b"
"#;

const DQUOTE_ESC1_EVENTS: &str = r#"
+DOC
=VAL "a/b
-DOC"#;

const DQUOTE_ESC2_INPUT: &str = r#"
"foo\nbar\\baz": 23"#;

const DQUOTE_ESC2_EVENTS: &str = r#"
+DOC
+MAP
=VAL "foo\nbar\\baz
=VAL :23
-MAP
-DOC"#;

const X1_NP9H_INPUT: &str = r#"
"folded 
to a space,	
 
to a line feed, or 	\
 \ 	non-content""#;

const X1_NP9H_EVENTS: &str = r#"
+DOC
=VAL "folded to a space,\nto a line feed, or \t \tnon-content
-DOC"#;

#[test]
fn dquote_escape() {
    assert_eq_event(X1_NP9H_INPUT, X1_NP9H_EVENTS);
    assert_eq_event(DQUOTE_ESC1_INPUT, DQUOTE_ESC1_EVENTS);
    assert_eq_event(DQUOTE_ESC2_INPUT, DQUOTE_ESC2_EVENTS);
    assert_eq_event(DQUOTE_STR_ESC1_INPUT, DQUOTE_STR_ESC_EVENTS);
}

const SQUOTE_STR1_INPUT: &str = r"
  'single quote'
    ";

const SQUOTE_STR2_INPUT: &str = r"
  'single
  quote'";

const SQUOTE_STR_EVENTS: &str = r"
+DOC
=VAL 'single quote
-DOC";

const SQUOTE_ESCAPE_INPUT: &str = r"'for single quote, use '' two of them'";
const SQUOTE_ESCAPE2_INPUT: &str = r"'for single quote, use
'' two of them'";
const SQUOTE_ESCAPE_EVENTS: &str = r"
+DOC
=VAL 'for single quote, use ' two of them
-DOC";

#[test]
fn quote_single() {
    assert_eq_event(SQUOTE_STR1_INPUT, SQUOTE_STR_EVENTS);
    assert_eq_event(SQUOTE_STR2_INPUT, SQUOTE_STR_EVENTS);
    assert_eq_event(SQUOTE_ESCAPE_INPUT, SQUOTE_ESCAPE_EVENTS);
    assert_eq_event(SQUOTE_ESCAPE2_INPUT, SQUOTE_ESCAPE_EVENTS);
}

const DQUOTE_STR1_INPUT: &str = r#"
  "double quote"
    "#;

const DQUOTE_STR2_INPUT: &str = r#"
  "double
  quote"
"#;

const DQUOTE_STR_EVENTS: &str = r#"
+DOC
=VAL "double quote
-DOC"#;

const DQUOTE_MULTI_INPUT: &str = r##"
 "test  
 
   tab" "##;

const DQUOTE_MULTI_EVENTS: &str = r#"
+DOC
=VAL "test\ntab
-DOC"#;

#[test]
fn dquote_solo() {
    assert_eq_event(DQUOTE_STR1_INPUT, DQUOTE_STR_EVENTS);
    assert_eq_event(DQUOTE_STR2_INPUT, DQUOTE_STR_EVENTS);
    assert_eq_event(DQUOTE_MULTI_INPUT, DQUOTE_MULTI_EVENTS);
}

const DQUOTE_MULTI1_INPUT: &str = r#"
  gen: "\
      foo\
      bar   
      baz "
"#;

const DQUOTE_MULTI1_EVENTS: &str = r#"
+DOC
+MAP
=VAL :gen
=VAL "foobar baz 
-MAP
-DOC"#;

const DQUOTE_MULTI2_INPUT: &str = r##"
 - "double   
             
 quote" "##;

const DQUOTE_MULTI2_EVENTS: &str = r#"
+DOC
+SEQ
=VAL "double\nquote
-SEQ
-DOC"#;

const X_6WPF_INPUT: &str = r#"
"
  baz
""#;

const X_6WPF_EVENTS: &str = r#"
+DOC
=VAL " baz 
-DOC"#;

#[test]
fn dquote_multiline() {
    assert_eq_event(DQUOTE_MULTI1_INPUT, DQUOTE_MULTI1_EVENTS);
    assert_eq_event(DQUOTE_MULTI2_INPUT, DQUOTE_MULTI2_EVENTS);
    assert_eq_event(X_6WPF_INPUT, X_6WPF_EVENTS);
}

const DQUOTE_END_INPUT: &str = r#"
"
---
""#;

const DQUOTE_END_EVENTS: &str = r#"
+DOC
ERR"#;

const DQUOTE_ERR2_INPUT: &str = r#"
"\c"
"#;

const DQUOTE_ERR2_EVENTS: &str = r#"
+DOC
ERR"#;

const DQUOTE_MISS_EOF_INPUT: &str = r#"
---
key: "missing

"#;

const DQUOTE_MISS_EOF_EVENTS: &str = r#"
+DOC ---
+MAP
=VAL :key
ERR"#;

const DQUOTE_INDENT_ERR_INPUT: &str = r#"
---
quoted: "a
b
c"

"#;

const DQUOTE_INDENT_ERR_EVENTS: &str = r#"
+DOC ---
+MAP
=VAL :quoted
ERR"#;

const DQUOTE_COMMENT_ERR_INPUT: &str = r##"
---
"quote"# invalid comment

"##;

const DQUOTE_COMMENT_ERR_EVENTS: &str = r#"
+DOC ---
=VAL "quote
ERR"#;

#[test]
fn dquote_err() {
    assert_eq_event(DQUOTE_END_INPUT, DQUOTE_END_EVENTS);
    assert_eq_event(DQUOTE_ERR2_INPUT, DQUOTE_ERR2_EVENTS);
    assert_eq_event(DQUOTE_MISS_EOF_INPUT, DQUOTE_MISS_EOF_EVENTS);
    assert_eq_event(DQUOTE_INDENT_ERR_INPUT, DQUOTE_INDENT_ERR_EVENTS);
    assert_eq_event(DQUOTE_COMMENT_ERR_INPUT, DQUOTE_COMMENT_ERR_EVENTS);
}

const DQUOTE_LEADING_TAB1_INPUT: &str = r#" "1 test
    \	tab" "#;

const DQUOTE_LEADING_TAB2_INPUT: &str = r#"
    "1 test
      \ttab" "#;

const DQUOTE_LEADING_TAB3_INPUT: &str = r#"
"1 test\t
    tab" "#;

const DQUOTE_LEADING_TAB4_INPUT: &str = r#"
    "1 test\t   
        tab" "#;

const DQUOTE_LEADING_TAB5_INPUT: &str = r#"
    "1 test\	
        tab"   "#;

const DQUOTE_LEADING_TAB_EVENTS: &str = r#"
+DOC
=VAL "1 test \ttab
-DOC"#;

const DQUOTE_LEADING_TAB2_EVENTS: &str = r#"
+DOC
=VAL "1 test\t tab
-DOC"#;

#[test]
fn dquote_trailing() {
    assert_eq_event(DQUOTE_LEADING_TAB1_INPUT, DQUOTE_LEADING_TAB_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB2_INPUT, DQUOTE_LEADING_TAB_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB3_INPUT, DQUOTE_LEADING_TAB2_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB4_INPUT, DQUOTE_LEADING_TAB2_EVENTS);
    assert_eq_event(DQUOTE_LEADING_TAB5_INPUT, DQUOTE_LEADING_TAB2_EVENTS);
}
const DQUOTE_EMPTY1_INPUT: &str = r"
a: '
  '
b: '  
  '
  ";
const DQUOTE_EMPTY1_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL ' 
=VAL :b
=VAL ' 
-MAP
-DOC";

#[test]
fn dquote_empty() {
    assert_eq_event(DQUOTE_EMPTY1_INPUT, DQUOTE_EMPTY1_EVENTS);
}

const X1_G4RS_INPUT: &str = r#"unicode: "Sosa did fine.\u263A""#;
const X2_G4RS_INPUT: &str = r#"unicode: "Sosa did fine.☺""#;
const X1_G4RS_EVENTS: &str = r#"
+DOC
+MAP
=VAL :unicode
=VAL "Sosa did fine.☺
-MAP
-DOC"#;

const X3_G4RS_INPUT: &str = r#"hex esc: "\x0d\x0a is \r\n""#;
const X3_G4RS_EVENTS: &str = r#"
+DOC
+MAP
=VAL :hex esc
=VAL "\r\n is \r\n
-MAP
-DOC"#;

#[test]
fn dquote_escape_unicode() {
    assert_eq_event(X3_G4RS_INPUT, X3_G4RS_EVENTS);
    assert_eq_event(X1_G4RS_INPUT, X1_G4RS_EVENTS);
    assert_eq_event(X2_G4RS_INPUT, X1_G4RS_EVENTS);
}
