pub const BLOCK1_INPUT: &str = r"
    - x
    - y
";
pub const BLOCK2_INPUT: &str = r"
- x
- y
";
pub const BLOCK_EVENTS: &str = r"
+DOC
+SEQ
=VAL :x
=VAL :y
-SEQ
-DOC";
pub const SEQ_PLAIN_INPUT: &str = r"
  - x
   - y
";
pub const SEQ_PLAIN2_INPUT: &str = r"
- x - y
";
pub const SEQ_PLAIN_EVENTS: &str = r"
+DOC
+SEQ
=VAL :x - y
-SEQ
-DOC";
pub const X1_33X3_INPUT: &str = r"
- !!int 1
- !!int -2
";
pub const X1_33X3_EVENTS: &str = r"
+DOC
+SEQ
=VAL <tag:yaml.org,2002:int> :1
=VAL <tag:yaml.org,2002:int> :-2
-SEQ
-DOC";
pub const BLOCK_ERR_INPUT: &str = r"
  - x
 - y
";
pub const BLOCK_ERR_EVENTS: &str = r"
+DOC
+SEQ
=VAL :x
-SEQ
ERR";
pub const WRONG_SEQ_INDENT_INPUT: &str = r"
a:
  - b
 - c
";
pub const WRONG_SEQ_INDENT_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
+SEQ
=VAL :b
-SEQ
ERR";
pub const SEQ_NO_MINUS_INPUT: &str = r"
map:
 - a
 c
";
pub const SEQ_NO_MINUS_EVENTS: &str = r"
+DOC
+MAP
=VAL :map
+SEQ
=VAL :a
ERR";
pub const X_9CWY_INPUT: &str = r"
key:
 - item1
 - item2
invalid
";
pub const X_9CWY_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
+SEQ
=VAL :item1
=VAL :item2
-SEQ
ERR";
pub const X_BD7L_INPUT: &str = r"
- item
invalid: x
";
pub const X_BD7L_EVENTS: &str = r"
+DOC
+SEQ
=VAL :item
ERR";
pub const X1_P2EQ_INPUT: &str = r"
- {}- invalid";
pub const X1_P2EQ_EVENTS: &str = r"
+DOC
+SEQ
+MAP
-MAP
ERR";
pub const X1_3ALJ_INPUT: &str = r"
 - - s1_i1
   - s1_i2
 - s2
";
pub const X2_3ALJ_INPUT: &str = r"
- - s1_i1
  - s1_i2
- s2
";
pub const X_3ALJ_EVENTS: &str = r"
+DOC
+SEQ
+SEQ
=VAL :s1_i1
=VAL :s1_i2
-SEQ
=VAL :s2
-SEQ
-DOC";
pub const BLOCK_NESTED_SEQ2_INPUT: &str = r"
  - - a
    - b
    - - c
  - d
";
pub const BLOCK_NESTED_SEQ2_EVENTS: &str = r"
+DOC
+SEQ
+SEQ
=VAL :a
=VAL :b
+SEQ
=VAL :c
-SEQ
-SEQ
=VAL :d
-SEQ
-DOC";
pub const FOLD_STR1_INPUT: &str = r"
  - >1-
   1
    2
   3
   4

";
pub const FOLD_STR1_EVENTS: &str = r"
+DOC
+SEQ
=VAL >1\n 2\n3 4
-SEQ
-DOC";
pub const FOLD_ERR_INPUT: &str = r"
 >
    
 invalid
";
pub const FOLD_ERR_EVENTS: &str = r"
+DOC
ERR";
pub const FOLD_ERR_EVENTS_SAPH: &str = r"
ERR";

pub const FOLD_STR2_INPUT: &str = r"
 >


  valid
";
pub const FOLD_STR2_EVENTS: &str = r"
+DOC
=VAL >\n\nvalid\n
-DOC";
pub const BLOCK_PLAIN_INPUT: &str = r"
  a
  b
  c
    d
  e
";
pub const BLOCK_PLAIN_EVENTS: &str = r"
+DOC
=VAL :a b c d e
-DOC";
pub const BLOCK_PLAIN2_INPUT: &str = r"
a
b
  c
d

e

";
pub const BLOCK_PLAIN2_EVENTS: &str = r"
+DOC
=VAL :a b c d\ne
-DOC";
pub const BLOCK_MULTI_INPUT: &str = r"
    word1
    # comment
    word2
";
pub const BLOCK_MULTI_EVENTS: &str = r"
+DOC
=VAL :word1
-DOC
ERR";
pub const BLOCK_FOLD_INPUT: &str = r"
>
 a
 b

 c


 d";
pub const BLOCK_FOLD_EVENTS: &str = r"
+DOC
=VAL >a b\nc\n\nd\n
-DOC";
pub const SIMPLE_FOLD1_INPUT: &str = r"
--- >1+";
pub const SIMPLE_FOLD2_INPUT: &str = r"
--- >1-";
pub const SIMPLE_FOLD_EVENTS: &str = r"
+DOC
=VAL >
-DOC";
pub const X1_X4QW_INPUT: &str = r"
test: |#comment";
pub const X1_X4QW_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
ERR";
pub const X2_X4QW_INPUT: &str = r"
test: |b";
pub const X2_X4QW_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
ERR";
pub const LITERAL1_INPUT: &str = r"
--- |1+ #tsts";
pub const LITERAL2_INPUT: &str = r"
--- |1-";
pub const SIMPLE_FOLDED_EVENTS: &str = r"
+DOC
=VAL |
-DOC";
pub const LIT_STR2_INPUT: &str = r"
strip: |-
  text
clip: |
  text
keep: |+
  text";
pub const LIT_STR2_EVENTS: &str = r"
+DOC
+MAP
=VAL :strip
=VAL |text
=VAL :clip
=VAL |text\n
=VAL :keep
=VAL |text\n
-MAP
-DOC";
pub const MULTILINE_PLAIN_INPUT: &str = r"
generic: !!str |
 test
 test
";
pub const MULTILINE_PLAIN_EVENTS: &str = r"
+DOC
+MAP
=VAL :generic
=VAL <tag:yaml.org,2002:str> |test\ntest\n
-MAP
-DOC";
pub const BLOCK_QUOTE_INPUT: &str = r#"
 plain:
   spans
   lines

 quoted:
   "text"
"#;
pub const BLOCK_QUOTE_EVENTS: &str = r#"
+DOC
+MAP
=VAL :plain
=VAL :spans lines
=VAL :quoted
=VAL "text
-MAP
-DOC"#;
pub const LITERAL3_INPUT: &str = r"
--- |+
 ab

  
...";
pub const LITERAL3_EVENTS: &str = r"
+DOC
=VAL |ab\n\n \n
-DOC";
pub const LITERAL_CHOMP_INPUT: &str = r"
Chomping: |
  Clipped

";
pub const LITERAL_CHOMP_EVENTS: &str = r"
+DOC
+MAP
=VAL :Chomping
=VAL |Clipped\n
-MAP
-DOC";
pub const LITERAL_ESCAPE_INPUT: &str = r"
block: |
  Hello\n";
pub const LITERAL_ESCAPE_EVENTS: &str = r"
+DOC
+MAP
=VAL :block
=VAL |Hello\\n\n
-MAP
-DOC";
pub const X1_Y79Y_000_INPUT: &str = r"
  foo: |
  			
			
";
pub const X1_Y79Y_000_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL |
-MAP
-DOC";
pub const X2_Y79Y_000_INPUT: &str = r"
foo:

bar: 1";
pub const X2_Y79Y_000_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL :
=VAL :bar
=VAL :1
-MAP
-DOC";
pub const X3_Y79Y_000_INPUT: &str = r"
foo: |
	
bar: 1";
pub const X3_Y79Y_000_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
ERR";
pub const X4_Y79Y_000_INPUT: &str = r"
  foo: |
  x";
pub const X4_Y79Y_000_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL |
ERR";
pub const LITERAL_ERR_INPUT: &str = r"
--- |0";
pub const LITERAL_ERR2_INPUT: &str = r"
--- |+10";
pub const SIMPLE_FOLDED_ERR_EVENTS: &str = r"
+DOC
ERR";
pub const X1_6VJK_INPUT: &str = r"
|
 XX
 X1

   Y1
   Y2

 Z3
";
pub const X1_6VJK_EVENTS: &str = r"
+DOC
=VAL |XX\nX1\n\n  Y1\n  Y2\n\nZ3\n
-DOC";
pub const X2_6VJK_INPUT: &str = r"
>
 X1

   Y1
   Y2

 Z3
";
pub const X2_6VJK_EVENTS: &str = r"
+DOC
=VAL >X1\n\n  Y1\n  Y2\n\nZ3\n
-DOC";
pub const X1_7T8X_INPUT: &str = r"
>
 line

 # Comment
";
pub const X1_7T8X_EVENTS: &str = r"
+DOC
=VAL >line\n# Comment\n
-DOC";
pub const X2_7T8X_INPUT: &str = r"
>
 line

# Comment
";
pub const X2_7T8X_EVENTS: &str = r"
+DOC
=VAL >line\n
-DOC";
pub const X1_JEF9_INPUT: &str = r"
- |+
   ";
pub const X1_JEF9_EVENTS: &str = r"
+DOC
+SEQ
=VAL |\n
-SEQ
-DOC";
pub const X1_F6MC_INPUT: &str = r"
a: >2
   more
  normal
";
pub const X2_F6MC_INPUT: &str = r"
b: >2


   more
  normal
";
pub const X1_F6MC_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL > more\nnormal\n
-MAP
-DOC";
pub const X2_F6MC_EVENTS: &str = r"
+DOC
+MAP
=VAL :b
=VAL >\n\n more\nnormal\n
-MAP
-DOC";
pub const PLAIN_MULTI_INPUT: &str = r"
1st line

 2nd non
    3rd non
";
pub const PLAIN_MULTI_EVENTS: &str = r"
+DOC
=VAL :1st line\n2nd non 3rd non
-DOC";
pub const X_8XDJ_INPUT: &str = r"
key: word1
#  xxx
  word2
";
pub const X_8XDJ_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
=VAL :word1
ERR";
pub const MAP_SIMPLE_INPUT: &str = r"
a: b
";
pub const MAP_SIMPLE2_INPUT: &str = r"
a:
  b
";
pub const MAP_SIMPLE_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :b
-MAP
-DOC";
pub const X1_1_SYW4_INPUT: &str = r"
hr:  65    # Home runs
avg: 0.278 # Batting average
";
pub const X1_2_SYW4_INPUT: &str = r"
hr:  65
avg: 0.278
";
pub const X1_SYW4_EVENTS: &str = r"
+DOC
+MAP
=VAL :hr
=VAL :65
=VAL :avg
=VAL :0.278
-MAP
-DOC";
pub const DQUOTE_MAP_INPUT: &str = r#"
quote: "a\/b"
"#;
pub const DQUOTE_MAP_EVENTS: &str = r#"
+DOC
+MAP
=VAL :quote
=VAL "a/b
-MAP
-DOC"#;
pub const DQUOTE_MUL_INPUT: &str = r#"
quoted: "multi
  line"
 "#;
pub const DQUOTE_MUL_EVENTS: &str = r#"
+DOC
+MAP
=VAL :quoted
=VAL "multi line
-MAP
-DOC"#;
pub const EMPTY_MAP_INPUT: &str = r"
:";
pub const EMPTY_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :
-MAP
-DOC";
pub const EMPTY_KEY_MAP2_INPUT: &str = r"
:
 a";
pub const EMPTY_KEY_MAP2_1_INPUT: &str = r"
: a";
pub const EMPTY_KEY_MAP2_EVENTS: &str = r#"
+DOC
+MAP
=VAL :
=VAL :a
-MAP
-DOC"#;
pub const MIX_EMPTY_MAP_INPUT: &str = r"
 a:
   x
   u
 c :
";
pub const MIX_EMPTY_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :x u
=VAL :c
=VAL :
-MAP
-DOC";
pub const MAP2_INPUT: &str = r"
:
a: b
: c
d:
";
pub const MAP2_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :
=VAL :a
=VAL :b
=VAL :
=VAL :c
=VAL :d
=VAL :
-MAP
-DOC";
pub const NESTED_EMPTY_INPUT: &str = r"
a :
 b:
  c:
d:";
pub const NESTED_EMPTY_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
+MAP
=VAL :b
+MAP
=VAL :c
=VAL :
-MAP
-MAP
=VAL :d
=VAL :
-MAP
-DOC";
pub const MULTI_EMPTY_INPUT: &str = r"
:
  :";
pub const MULTI_EMPTY_EVENTS: &str = r"
+DOC
+MAP
=VAL :
+MAP
=VAL :
=VAL :
-MAP
-MAP
-DOC";
pub const X1_6KGN_INPUT: &str = r"
a: &anchor
b: *anchor";
pub const X1_6KGN_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL &anchor :
=VAL :b
=ALI *anchor
-MAP
-DOC";
pub const X1_NKF9_INPUT: &str = r"
---
# empty key and value
:
---";
pub const X1_NKF9_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :
-MAP
-DOC
+DOC
=VAL :
-DOC";
pub const MULTILINE_COMMENT1_INPUT: &str = r"
  mul:
    abc  # a comment
";
pub const MULTILINE_COMMENT1_2_INPUT: &str = r"
  mul  :
    abc  # a comment
";
pub const MULTILINE_COMMENT1_EVENTS: &str = r"
+DOC
+MAP
=VAL :mul
=VAL :abc
-MAP
-DOC";
pub const MULTILINE_COMMENT2_INPUT: &str = r"
  multi:
    ab  # a comment
    xyz  # a commeent
";
pub const MULTILINE_COMMENT2_EVENTS: &str = r"
+DOC
+MAP
=VAL :multi
=VAL :ab
ERR";
pub const MULTILINE_COMMENT3_INPUT: &str = r"
  multi:
    ab
    xyz  # a commeent
";
pub const MULTILINE_COMMENT3_EVENTS: &str = r"
+DOC
+MAP
=VAL :multi
=VAL :ab xyz
-MAP
-DOC";
pub const EXP_MAP_INPUT: &str = r"
  ? test
  : value
";
pub const EXP_BLOCK_MAP_MIX_INPUT: &str = r"
  ? test
  : value
  tx: x
";
pub const EXP_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
=VAL :value
-MAP
-DOC";
pub const EXP_BLOCK_MAP_MIX_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
=VAL :value
=VAL :tx
=VAL :x
-MAP
-DOC";
pub const EXP_MAP_FOLD_INPUT: &str = r"
 ? >
   test
 : x
";
pub const EXP_MAP_FOLD_EVENTS: &str = r"
+DOC
+MAP
=VAL >test\n
=VAL :x
-MAP
-DOC";
pub const EXP_MAP_COMP_INPUT: &str = r"
---
?
- a
- b
:
- c";
pub const EXP_MAP_COMP_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :a
=VAL :b
-SEQ
+SEQ
=VAL :c
-SEQ
-MAP
-DOC";
pub const X_7W2P_INPUT: &str = r"
? a
? b
c:
";
pub const X_7W2P_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :
=VAL :b
=VAL :
=VAL :c
=VAL :
-MAP
-DOC";
pub const X_5WE3_INPUT: &str = r"

? explicit key # Empty value
? |
  block key";
pub const X_5WE3_EVENTS: &str = r"
+DOC
+MAP
=VAL :explicit key
=VAL :
=VAL |block key\n
=VAL :
-MAP
-DOC";
pub const X1_A2M4_INPUT: &str = r"
? a
: -	b";
pub const X1_A2M4_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
+SEQ
=VAL :b
-SEQ
-MAP
-DOC";
pub const X1_2XXW_INPUT: &str = r"
--- !!set
? Mark McGwire";
pub const X1_2XXW_EVENTS: &str = r"
+DOC
+MAP <tag:yaml.org,2002:set>
=VAL :Mark McGwire
=VAL :
-MAP
-DOC";
pub const X1_V9D5_INPUT: &str = r"
- ? earth: blue
  : moon: white
";
pub const X1_V9D5_EVENTS: &str = r"
+DOC
+SEQ
+MAP
+MAP
=VAL :earth
=VAL :blue
-MAP
+MAP
=VAL :moon
=VAL :white
-MAP
-MAP
-SEQ
-DOC";
pub const EXP_MAP_EMPTY_INPUT: &str = r"
  # Sets are represented as a
---
? a
? b
? c
";
pub const EXP_MAP_EMPTY_INPUT_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :
=VAL :b
=VAL :
=VAL :c
=VAL :
-MAP
-DOC";
pub const EXP_MAP_FAKE_EMPTY_INPUT: &str = r"
  ? x
   ? x
";
pub const EXP_MAP_FAKE_EMPTY_EVENTS: &str = r"
+DOC
+MAP
=VAL :x ? x
=VAL :
-MAP
-DOC";
pub const EMPTY_KEY_MAP_INPUT: &str = r"
: a
: b
";
pub const EMPTY_KEY_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :a
=VAL :
=VAL :b
-MAP
-DOC";
pub const EXP_BLOCK_MAP_ERR1: &str = r"
   ? test
  : value
";
pub const EXP_BLOCK_MAP_ERR1_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
ERR";
pub const EXP_BLOCK_MAP_ERR2: &str = r"
 ? test
  : value
";
pub const EXP_BLOCK_MAP_ERR2_EVENTS: &str = r"
+DOC
+MAP
=VAL :test
ERR";
pub const INLINE_ERR_INPUT: &str = r"
 a: b:
";
pub const INLINE_ERR_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
ERR";
pub const ERR_MULTILINE_KEY_INPUT: &str = "
invalid
 key :  x";
pub const ERR_MULTILINE_KEY_EVENTS: &str = "
+DOC
ERR";
pub const ERR_INVALID_KEY1_INPUT: &str = "
a:
  b
c";
pub const ERR_INVALID_KEY1_EVENTS: &str = "
+DOC
+MAP
=VAL :a
=VAL :b
ERR";
pub const ERR_INVALID_KEY2_INPUT: &str = r#"
 a:
   b
 "c
  x""#;
pub const ERR_INVALID_KEY2_EVENTS: &str = r#"
+DOC
+MAP
=VAL :a
=VAL :b
ERR"#;
pub const ERR_INVALID_KEY3_INPUT: &str = r"
top1:
  key1: val1
top2
";
pub const ERR_INVALID_KEY3_EVENTS: &str = r"
+DOC
+MAP
=VAL :top1
+MAP
=VAL :key1
=VAL :val1
-MAP
ERR";
pub const ERR_TRAIL_INPUT: &str = r#"
'key': "quote" trail
"#;
pub const ERR_TRAIL_EVENTS: &str = r#"
+DOC
+MAP
=VAL 'key
=VAL "quote
ERR"#;
pub const COMPLEX_KEYS_INPUT: &str = r##"
a!"#$%&'()*+,-./09:;<=>?@AZ[\]^_`az{|}~: safe
:foo: baz
-foo: boo
"##;
pub const COMPLEX_KEYS_EVENTS: &str = r##"
+DOC
+MAP
=VAL :a!"#$%&'()*+,-./09:;<=>?@AZ[\\]^_`az{|}~
=VAL :safe
=VAL ::foo
=VAL :baz
=VAL :-foo
=VAL :boo
-MAP
-DOC"##;
pub const COMPLEX_NESTED_INPUT: &str = r"
not:
  two: [
    nest
   ]
  ";
pub const COMPLEX_NESTED_EVENTS: &str = r"
+DOC
+MAP
=VAL :not
+MAP
=VAL :two
+SEQ
=VAL :nest
-SEQ
-MAP
-MAP
-DOC";
pub const NESTED_INPUT: &str = r"
---
hr: # 1998 hr ranking
  - Mark McGwire
  - Sammy Sosa
";
pub const NESTED_EVENTS: &str = r"
+DOC
+MAP
=VAL :hr
+SEQ
=VAL :Mark McGwire
=VAL :Sammy Sosa
-SEQ
-MAP
-DOC";
pub const X1_9C9N_INPUT: &str = r"
flow: [a,
b,
 c]";
pub const X1_9C9N_EVENTS: &str = r"
+DOC
+MAP
=VAL :flow
+SEQ
=VAL :a
ERR";
pub const MAP_AND_COMMENT_INPUT: &str = r"
hr:
  - aaa
  # comment
  - &xx bbb
";
pub const MAP_AND_COMMENT_EVENTS: &str = r"
+DOC
+MAP
=VAL :hr
+SEQ
=VAL :aaa
=VAL &xx :bbb
-SEQ
-MAP
-DOC";
pub const X_7ZZ5_INPUT: &str = r"
key2: {}
";
pub const X_7ZZ5_EVENTS: &str = r"
+DOC
+MAP
=VAL :key2
+MAP
-MAP
-MAP
-DOC";
pub const X1_87E4_INPUT: &str = r"
'implicit block key' : [
  'implicit flow key' : value,
 ]
";
pub const X2_87E4_INPUT: &str = r"
'implicit block key' : [
  'implicit flow key' : value
 ]
";
pub const X_87E4_EVENTS: &str = r"
+DOC
+MAP
=VAL 'implicit block key
+SEQ
+MAP
=VAL 'implicit flow key
=VAL :value
-MAP
-SEQ
-MAP
-DOC";
pub const X_8KB6_INPUT: &str = r"
- { multi
  line, a: b}
";
pub const X_8KB6_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL :multi line
=VAL :
=VAL :a
=VAL :b
-MAP
-SEQ
-DOC";
pub const X1_6HB6_INPUT: &str = r"
Not indented:
 Flow style: [       # comment
   By two,           # comment
  Also by two,       # comment
  	Still by two     # comment
    ]      # comment
";
pub const X1_6HB6_EVENTS: &str = r"
+DOC
+MAP
=VAL :Not indented
+MAP
=VAL :Flow style
+SEQ
=VAL :By two
=VAL :Also by two
=VAL :Still by two
-SEQ
-MAP
-MAP
-DOC";
pub const X1_4AW9_INPUT: &str = r"
- aaa: |2
    xxx";
pub const X1_4AW9_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL :aaa
=VAL |xxx\n
-MAP
-SEQ
-DOC";
pub const MAPS_WITH_QUOTES_INPUT: &str = r#"
"double" :
  'single'  :   &alias plain
"#;
pub const MAPS_WITH_QUOTES_EVENTS: &str = r#"
+DOC
+MAP
=VAL "double
+MAP
=VAL 'single
=VAL &alias :plain
-MAP
-MAP
-DOC"#;
pub const NESTED_MAPS_INPUT: &str = r#"
"top1" :
  'key1' :
    down : &x1 test
'top2' :
  *x1 :  scalar2
"#;
pub const NESTED_MAPS_EVENTS: &str = r#"
+DOC
+MAP
=VAL "top1
+MAP
=VAL 'key1
+MAP
=VAL :down
=VAL &x1 :test
-MAP
-MAP
=VAL 'top2
+MAP
=ALI *x1
=VAL :scalar2
-MAP
-MAP
-DOC"#;
pub const X1_Q9WF_INPUT: &str = r"
{}:
  hr: a";
pub const X1_Q9WF_EVENTS: &str = r"
+DOC
+MAP
+MAP
-MAP
+MAP
=VAL :hr
=VAL :a
-MAP
-MAP
-DOC";
pub const ALIAS_N_MAPS_INPUT: &str = r#"
"top1" : &node
  &x1 'key1' : 'val'

'top2' :
  *x1 :  scalar2
"#;
pub const ALIAS_N_MAPS_EVENTS: &str = r#"
+DOC
+MAP
=VAL "top1
+MAP &node
=VAL &x1 'key1
=VAL 'val
-MAP
=VAL 'top2
+MAP
=ALI *x1
=VAL :scalar2
-MAP
-MAP
-DOC"#;
pub const ALIAS_N_MAPS2_INPUT: &str = r"
top3: &alias1
  *alias1 : scalar3
 ";
pub const ALIAS_N_MAPS2_EVENTS: &str = r"
+DOC
+MAP
=VAL :top3
+MAP &alias1
=ALI *alias1
=VAL :scalar3
-MAP
-MAP
-DOC";
pub const ALIAS_N_COMP_MAP_INPUT: &str = r"
&map
&key [ &item a, b]: value
";
pub const ALIAS_N_COMP_MAP_EVENTS: &str = r"
+DOC
+MAP &map
+SEQ &key
=VAL &item :a
=VAL :b
-SEQ
=VAL :value
-MAP
-DOC";
pub const X1_SR86_INPUT: &str = r"
&b *a";
pub const X1_SR86_EVENTS: &str = r"
+DOC
ERR";
pub const X1_PW8X_INPUT: &str = r"
-
  ? &d
-
  ? &e
  : &a";
pub const X1_PW8X_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL &d :
=VAL :
-MAP
+MAP
=VAL &e :
=VAL &a :
-MAP
-SEQ
-DOC";
pub const X2_PW8X_INPUT: &str = r"
-
  ? &d";
pub const X2_PW8X_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL &d :
=VAL :
-MAP
-SEQ
-DOC";
pub const X3_PW8X_INPUT: &str = r"
-
  ? &e
  : &a";
pub const X3_PW8X_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL &e :
=VAL &a :
-MAP
-SEQ
-DOC";
pub const ALIAS_N_SEQ1_INPUT: &str = r"
&seq
 - a
 ";
pub const ALIAS_N_SEQ1_EVENTS: &str = r"
+DOC
+SEQ &seq
=VAL :a
-SEQ
-DOC";
pub const ALIAS_N_SEQ2_INPUT: &str = r"
 &seq  - a
  ";
pub const ALIAS_N_SEQ2_EVENTS: &str = r"
+DOC
ERR";
pub const ALIAS_N_SEQ3_INPUT: &str = r"
  - &node a
  ";
pub const ALIAS_N_SEQ3_EVENTS: &str = r"
+DOC
+SEQ
=VAL &node :a
-SEQ
-DOC";
pub const X1_G9HC_INPUT: &str = r"
 seq:
 &anchor
 - a";
pub const X1_G9HC_EVENTS: &str = r"
+DOC
+MAP
=VAL :seq
ERR";
pub const X2_1_G9HC_INPUT: &str = r"
seq:
 &anchor
 - a";
pub const X2_2_G9HC_INPUT: &str = r"
 seq:
  	&anchor
  - a";
pub const X2_G9HC_EVENTS: &str = r"
+DOC
+MAP
=VAL :seq
+SEQ &anchor
=VAL :a
-SEQ
-MAP
-DOC";
pub const X1_HMQ5_INPUT: &str = r#"
!!str &a1 "foo":
  !!str bar
&a2 baz : *a1
"#;
pub const X1_HMQ5_EVENTS: &str = r#"
+DOC
+MAP
=VAL &a1 <tag:yaml.org,2002:str> "foo
=VAL <tag:yaml.org,2002:str> :bar
=VAL &a2 :baz
=ALI *a1
-MAP
-DOC"#;
pub const X1_57H4_INPUT: &str = r"
  sequence: !!seq
  - a
  - !!str
    - b
  mapping: !!map
    foo: bar
";
pub const X1_57H4_EVENTS: &str = r"
+DOC
+MAP
=VAL :sequence
+SEQ <tag:yaml.org,2002:seq>
=VAL :a
+SEQ <tag:yaml.org,2002:str>
=VAL :b
-SEQ
-SEQ
=VAL :mapping
+MAP <tag:yaml.org,2002:map>
=VAL :foo
=VAL :bar
-MAP
-MAP
-DOC";
pub const X2_57H4_INPUT: &str = r"
  - !!str
    - b";
pub const X2_57H4_EVENTS: &str = r"
+DOC
+SEQ
+SEQ <tag:yaml.org,2002:str>
=VAL :b
-SEQ
-SEQ
-DOC";
pub const X3_57H4_INPUT: &str = r"
  sequence: !!seq
  - a";
pub const X3_57H4_EVENTS: &str = r"
+DOC
+MAP
=VAL :sequence
+SEQ <tag:yaml.org,2002:seq>
=VAL :a
-SEQ
-MAP
-DOC";
pub const TAG_DEF_INPUT: &str = r"
 ! test
";
pub const TAG_DEF_EVENTS: &str = r"
+DOC
=VAL <!> :test
-DOC";
pub const EXP_TAG_INPUT: &str = r"
!<tag:yaml.org,2002:str> foo :
  !<!bar> baz";
pub const EXP_TAG_EVENTS: &str = r"
+DOC
+MAP
=VAL <tag:yaml.org,2002:str> :foo
=VAL <!bar> :baz
-MAP
-DOC";
pub const ANCHOR_COLON_INPUT: &str = r"
&node3:  key : scalar3
*node3: : x";
pub const ANCHOR_COLON_EVENTS: &str = r"
+DOC
+MAP
=VAL &node3: :key
=VAL :scalar3
=ALI *node3:
=VAL :x
-MAP
-DOC";
pub const ANCHOR_ERR_INPUT: &str = r"
top2: &node2
  &v2 val";
pub const ANCHOR_ERR_EVENTS: &str = r"
+DOC
+MAP
=VAL :top2
ERR";
pub const X1_735Y_INPUT: &str = r"
- >
 Block scalar
- !!map # Block collection
  foo : bar
";
pub const X1_735Y_EVENTS: &str = r"
+DOC
+SEQ
=VAL >Block scalar\n
+MAP <tag:yaml.org,2002:map>
=VAL :foo
=VAL :bar
-MAP
-SEQ
-DOC";
pub const MIX_BLOCK_INPUT: &str = r"
-
  key: x
  val: 8
-
  val: y
";
pub const MIX_BLOCK_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL :key
=VAL :x
=VAL :val
=VAL :8
-MAP
+MAP
=VAL :val
=VAL :y
-MAP
-SEQ
-DOC";
pub const MIX2_BLOCK_INPUT: &str = r"
  sequence:
  - a
  mapping:
   foo: bar
 ";
pub const MIX2_BLOCK_EVENTS: &str = r"
+DOC
+MAP
=VAL :sequence
+SEQ
=VAL :a
-SEQ
=VAL :mapping
+MAP
=VAL :foo
=VAL :bar
-MAP
-MAP
-DOC";
pub const TAG1_1_INPUT: &str = r"
  !!str a";
pub const TAG1_2_INPUT: &str = r"
  !!str
  a";
pub const TAG1_EVENTS: &str = r"
+DOC
=VAL <tag:yaml.org,2002:str> :a
-DOC";
pub const COMPLEX_TAG2_INPUT: &str = r"
- !!str c
--- !!str
d
e";
pub const COMPLEX_TAG2_EVENTS: &str = r"
+DOC
+SEQ
=VAL <tag:yaml.org,2002:str> :c
-SEQ
-DOC
+DOC
=VAL <tag:yaml.org,2002:str> :d e
-DOC";
pub const X_74H7_INPUT: &str = r"
!!str a: b
c: !!int 42
!!str 23: !!bool false";
pub const X_74H7_EVENTS: &str = r"
+DOC
+MAP
=VAL <tag:yaml.org,2002:str> :a
=VAL :b
=VAL :c
=VAL <tag:yaml.org,2002:int> :42
=VAL <tag:yaml.org,2002:str> :23
=VAL <tag:yaml.org,2002:bool> :false
-MAP
-DOC";
pub const MULTI_LINE_INPUT: &str = r"
x: a
 b

 c";
pub const MULTI_LINE_EVENTS: &str = r"
+DOC
+MAP
=VAL :x
=VAL :a b\nc
-MAP
-DOC";
pub const MULTI_LINE_SEQ_INPUT: &str = r"
- a
 b

 c";
pub const MULTI_LINE_SEQ_EVENTS: &str = r"
+DOC
+SEQ
=VAL :a b\nc
-SEQ
-DOC";
pub const X_BF9H_INPUT: &str = r"
plain:  a
        b # comment
        c
";
pub const X_BF9H_EVENTS: &str = r"
+DOC
+MAP
=VAL :plain
=VAL :a b
ERR";
pub const X_BS4K_INPUT: &str = r"
line1 # comment
line2";
pub const X_BS4K_EVENTS: &str = r"
+DOC
=VAL :line1
-DOC
ERR";
pub const SEQ_SAME_LINE_INPUT: &str = r"
  key: - a
";
pub const SEQ_SAME_LINE_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
ERR";
pub const X1_S7BG_INPUT: &str = r"
- :,";
pub const X1_S7BG_EVENTS: &str = r"
+DOC
+SEQ
=VAL ::,
-SEQ
-DOC";
pub const TAG_SHORT_INPUT: &str = "
%TAG !e! tag:example.com,2000:app/
---
- !local foo
- !!str bar
- !e!tag%21 baz";
pub const TAG_SHORT_EVENTS: &str = "
+DOC
+SEQ
=VAL <!local> :foo
=VAL <tag:yaml.org,2002:str> :bar
=VAL <tag:example.com,2000:app/tag!> :baz
-SEQ
-DOC";
pub const X1_TAG_SHORT_INPUT: &str = "
---
- !local foo
- !!str bar
";
pub const X1_TAG_SHORT_EVENTS: &str = "
+DOC
+SEQ
=VAL <!local> :foo
=VAL <tag:yaml.org,2002:str> :bar
-SEQ
-DOC";
pub const X1_QLJ7_INPUT: &str = r"
%TAG !prefix! tag:example.com,2011:
--- !prefix!A
a: b
--- !prefix!B
c: d";
pub const X1_QLJ7_EVENTS: &str = r"
+DOC
+MAP <tag:example.com,2011:A>
=VAL :a
=VAL :b
-MAP
-DOC
+DOC
ERR";
pub const X1_U99R_INPUT: &str = r"
!!str, x";
pub const X1_U99R_EVENTS: &str = r"
+DOC
ERR";
pub const X2_U99R_INPUT: &str = r"
[!!str, xxx]";
pub const X2_U99R_EVENTS: &str = r"
+DOC
+SEQ
=VAL <tag:yaml.org,2002:str> :
=VAL :xxx
-SEQ
-DOC";
pub const X1_9KAX_INPUT: &str = r"
&a1
!!str
scalar1";
pub const X2_9KAX_INPUT: &str = r"
!!str
&a1
scalar1";
pub const X1_9KAX_EVENTS: &str = r"
+DOC
=VAL &a1 <tag:yaml.org,2002:str> :scalar1
-DOC";
pub const X3_9KAX_INPUT: &str = r"
&a4 !!map
&a5 !!str key5: value4
";
pub const X3_9KAX_EVENTS: &str = r"
+DOC
+MAP &a4 <tag:yaml.org,2002:map>
=VAL &a5 <tag:yaml.org,2002:str> :key5
=VAL :value4
-MAP
-DOC";
pub const X4_9KAX_INPUT: &str = r"
a6: 1
&anchor6 b6: 2";
pub const X4_9KAX_EVENTS: &str = r"
+DOC
+MAP
=VAL :a6
=VAL :1
=VAL &anchor6 :b6
=VAL :2
-MAP
-DOC";
pub const X5_9KAX_INPUT: &str = r"
!!map
&a8 !!str key8: value7";
pub const X5_9KAX_EVENTS: &str = r"
+DOC
+MAP <tag:yaml.org,2002:map>
=VAL &a8 <tag:yaml.org,2002:str> :key8
=VAL :value7
-MAP
-DOC";
pub const X1_6JWB_INPUT: &str = r"
foo:
  - !!map
    k: v
";
pub const X1_6JWB_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
+SEQ
+MAP <tag:yaml.org,2002:map>
=VAL :k
=VAL :v
-MAP
-SEQ
-MAP
-DOC";
pub const X1_DK95_INPUT: &str = r"
foo: 'bar
	baz'";
pub const X1_DK95_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
ERR";
pub const X2_DK95_INPUT: &str = r"
foo:
	bar: baz";
pub const X2_DK95_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
ERR";
pub const X3_DK95_INPUT: &str = r"
foo:
  a: 1
  	b: 2";
pub const X3_DK95_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
+MAP
=VAL :a
=VAL :1
ERR";
pub const X1_DMG6_INPUT: &str = r"
key:
  ok: 1
 wrong: 2";
pub const X1_DMG6_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
+MAP
=VAL :ok
=VAL :1
-MAP
ERR";
pub const X1_EW3V_INPUT: &str = r"
k1: v1
  key2: v2
";
pub const X1_EW3V_EVENTS: &str = r"
+DOC
+MAP
=VAL :k1
ERR";
pub const X1_7LBH_INPUT: &str = r#"
a: b
"c
 d": 1"#;
pub const X1_7LBH_EVENTS: &str = r#"
+DOC
+MAP
=VAL :a
=VAL :b
ERR"#;
pub const X1_U44R_INPUT: &str = r#"
map:
  a: "1"
   b: "2"
"#;
pub const X1_U44R_EVENTS: &str = r#"
+DOC
+MAP
=VAL :map
+MAP
=VAL :a
=VAL "1
ERR"#;
pub const SEQ_EMPTY1_INPUT: &str = r"
-
";
pub const SEQ_EMPTY1_EVENTS: &str = r"
+DOC
+SEQ
=VAL :
-SEQ
-DOC";
pub const SEQ_EMPTY2_INPUT: &str = r"
-
- ";
pub const SEQ_EMPTY2_EVENTS: &str = r"
+DOC
+SEQ
=VAL :
=VAL :
-SEQ
-DOC";
pub const X1_DC7X_INPUT: &str = r"
seq:
  - a	";
pub const X1_DC7X_EVENTS: &str = r"
+DOC
+MAP
=VAL :seq
+SEQ
=VAL :a
-SEQ
-MAP
-DOC";
pub const X1_Y79Y_001_INPUT: &str = r"
foo: |
 	
bar: 1";
pub const X1_Y79Y_001_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL |\t\n
=VAL :bar
=VAL :1
-MAP
-DOC";
pub const X1_Y79Y_004_INPUT: &str = r"
- -";
pub const X1_Y79Y_004_EVENTS: &str = r"
+DOC
+SEQ
+SEQ
=VAL :
-SEQ
-SEQ
-DOC";
pub const X2_Y79Y_004_INPUT: &str = r"
-	-";
pub const X2_Y79Y_004_EVENTS: &str = r"
+DOC
+SEQ
ERR";
pub const X3_Y79Y_004_INPUT: &str = r"
-	-
 ";
pub const X1_Y79Y_006_INPUT: &str = r"
?	-";
pub const X1_Y79Y_006_EVENTS: &str = r"
+DOC
+MAP
ERR";
pub const X2_Y79Y_006_INPUT: &str = r"
? -";
pub const X2_Y79Y_006_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :
-SEQ
=VAL :
-MAP
-DOC";
pub const X3_Y79Y_006_INPUT: &str = r"
? -
";
pub const X3_Y79Y_006_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :
-SEQ
=VAL :
-MAP
-DOC";
pub const X1_Y79Y_007_INPUT: &str = r"
? -
:	-
";
pub const X1_Y79Y_007_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :
-SEQ
ERR";
pub const X1_Y79Y_009_INPUT: &str = r"
? key:
:	foo:
";
pub const X1_Y79Y_009_EVENTS: &str = r"
+DOC
+MAP
+MAP
=VAL :key
=VAL :
-MAP
ERR";
pub const X2_Y79Y_009_INPUT: &str = r"
? key:
";
pub const X2_Y79Y_009_EVENTS: &str = r"
+DOC
+MAP
+MAP
=VAL :key
=VAL :
-MAP
=VAL :
-MAP
-DOC";
pub const X3_Y79Y_009_INPUT: &str = r"
? key:
: ";
pub const X3_Y79Y_009_EVENTS: &str = r"
+DOC
+MAP
+MAP
=VAL :key
=VAL :
-MAP
=VAL :
-MAP
-DOC";
pub const X1_FH7J_INPUT: &str = r"
- !!str
- !!null : a";
pub const X1_FH7J_EVENTS: &str = r"
+DOC
+SEQ
=VAL <tag:yaml.org,2002:str> :
+MAP
=VAL <tag:yaml.org,2002:null> :
=VAL :a
-MAP
-SEQ
-DOC";
pub const X2_FH7J_INPUT: &str = r"
- !!str : !!null";
pub const X2_FH7J_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL <tag:yaml.org,2002:str> :
=VAL <tag:yaml.org,2002:null> :
-MAP
-SEQ
-DOC";
pub const X3_FH7J_INPUT: &str = r"
-
  b: !!str
- x";
pub const X3_FH7J_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL :b
=VAL <tag:yaml.org,2002:str> :
-MAP
=VAL :x
-SEQ
-DOC";
pub const X4_FH7J_INPUT: &str = r"
- !!str
- !!str : !!null
";
pub const X4_FH7J_EVENTS: &str = r"
+DOC
+SEQ
=VAL <tag:yaml.org,2002:str> :
+MAP
=VAL <tag:yaml.org,2002:str> :
=VAL <tag:yaml.org,2002:null> :
-MAP
-SEQ
-DOC";
pub const X1_UKK6_02_INPUT: &str = r"
!
";
pub const X1_UKK6_02_EVENTS: &str = r"
+DOC
=VAL <!> :
-DOC";
pub const X1_K858_INPUT: &str = "
strip: >-

clip: >

keep: |+

";
pub const X1_K858_EVENTS: &str = r"
+DOC
+MAP
=VAL :strip
=VAL >
=VAL :clip
=VAL >
=VAL :keep
=VAL |\n
-MAP
-DOC";
pub const X1_MJS9_INPUT: &str = r"
>
  a 

  	 b

  c
";
pub const X1_MJS9_EVENTS: &str = r"
+DOC
=VAL >a \n\n\t b\n\nc\n
-DOC";
pub const X1_KK5P_INPUT: &str = "
complex1:
  ? - a";
pub const X1_KK5P_EVENTS: &str = "
+DOC
+MAP
=VAL :complex1
+MAP
+SEQ
=VAL :a
-SEQ
=VAL :
-MAP
-MAP
-DOC";
pub const X2_KK5P_INPUT: &str = "
complex5:
  ? - a
  : - b";
pub const X2_KK5P_EVENTS: &str = r"
+DOC
+MAP
=VAL :complex5
+MAP
+SEQ
=VAL :a
-SEQ
+SEQ
=VAL :b
-SEQ
-MAP
-MAP
-DOC";
pub const X1_M5DY_INPUT: &str = r"
? [ New York Yankees,
    Atlanta Braves ]
: [ 2001-07-02, 2001-08-12,
    2001-08-14 ]
";
pub const X1_M5DY_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :New York Yankees
=VAL :Atlanta Braves
-SEQ
+SEQ
=VAL :2001-07-02
=VAL :2001-08-12
=VAL :2001-08-14
-SEQ
-MAP
-DOC";
pub const X2_M5DY_INPUT: &str = r"
? a
:
  - c1

? b
: c";
pub const X2_M5DY_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
+SEQ
=VAL :c1
-SEQ
=VAL :b
=VAL :c
-MAP
-DOC";
pub const X1_RZP5_INPUT: &str = r#"
? # lala
 - seq1

"#;
pub const X1_RZP5_EVENTS: &str = r#"
+DOC
+MAP
+SEQ
=VAL :seq1
-SEQ
=VAL :
-MAP
-DOC"#;
pub const EMPTY_DOC_ERR_INPUT: &str = r#"
# test"
  # test
%YAML 1.3 #arst
"#;
pub const EMPTY_DOC_ERR_EVENTS: &str = r"
%YAML 1.3
ERR";
pub const EMPTY_DOC_INPUT: &str = r"
%YAML 1.2
---
";

pub const EMPTY_DOC_EVENTS_SAPH: &str = r"
+DOC
=VAL :
-DOC";
pub const EMPTY_DOC_EVENTS: &str = r"
%YAML 1.2
+DOC
=VAL :
-DOC";
pub const DOC_EMPTY_TAG_INPUT: &str = r"
%YAM 1.2
---
";
pub const DOC_EMPTY_TAG_EVENTS: &str = r"
+DOC
=VAL :
-DOC";
pub const ERR_DIRECTIVE1_INPUT: &str = r"
%YAML 1.2
...
";
pub const ERR_DIRECTIVE1_EVENTS: &str = r"
ERR
%YAML 1.2
+DOC
-DOC";
pub const ERR_DIRECTIVE2_INPUT: &str = r"
%YAML 1.2#err
...
";
pub const ERR_DIRECTIVE2_EVENTS: &str = r"
ERR
ERR
%YAML 1.2
+DOC
-DOC";
pub const ERR_DIRECTIVE3_INPUT: &str = r"
%YAML 1.2 err
---
";
pub const ERR_DIRECTIVE3_EVENTS: &str = r"
ERR
%YAML 1.2
+DOC
=VAL :
-DOC";
pub const ERR_MULTIDOC_INPUT: &str = r"
%YAML 1.2
---
%YAML 1.2
---
";
pub const ERR_MULTIDOC_EVENTS: &str = r"
%YAML 1.2
+DOC
ERR
-DOC
ERR
%YAML 1.2
+DOC
=VAL :
-DOC";
pub const ERR_DIRECTIVE4_INPUT: &str = r"%YAML 1.1#...
---";
pub const ERR_DIRECTIVE4_EVENTS: &str = r"
ERR
%YAML 1.1
+DOC
=VAL :
-DOC";
pub const SIMPLE_DOC_INPUT: &str = r"
---[]";
pub const SIMPLE_DOC_EVENTS: &str = r"
+DOC
+SEQ
-SEQ
-DOC";
pub const SIMPLE_DOC2_INPUT: &str = r#"
%YAML 1.3 #comment
          #comment
---
"test"
"#;
pub const SIMPLE_DOC2_EVENTS: &str = r#"
%YAML 1.3
+DOC
=VAL "test
-DOC"#;
pub const EMPTY1_INPUT: &str = r"
---
...
";
pub const EMPTY2_INPUT: &str = r"
---
# comment
...";
pub const EMPTY_EVENTS: &str = r"
+DOC
=VAL :
-DOC";
pub const NO_DOC_INPUT: &str = "\n...\n";
pub const NO_DOC_EVENTS: &str = "";
pub const FOOTER_INPUT: &str = r#"
"test"
...
"#;
pub const FOOTER_EVENTS: &str = r#"
+DOC
=VAL "test
-DOC"#;
pub const POST_DOC_ERR_INPUT: &str = r"
---
... invalid
";
pub const POST_DOC_ERR_EVENTS: &str = r"
+DOC
=VAL :
-DOC
ERR
=VAL :invalid
-DOC";
pub const MULTI_DOC1_INPUT: &str = r"
---
? a
: b
---
- c
";
pub const MULTI_DOC1_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :b
-MAP
-DOC
+DOC
+SEQ
=VAL :c
-SEQ
-DOC";
pub const X1_6ZKB_INPUT: &str = r"
Document
---
# Empty
...";
pub const X1_6ZKB_EVENTS: &str = r#"
+DOC
=VAL :Document
-DOC
+DOC
=VAL :
-DOC"#;
pub const MULTI_DOC3_INPUT: &str = r"
---
---";
pub const MULTI_DOC3_EVENTS: &str = r"
+DOC
=VAL :
-DOC
+DOC
=VAL :
-DOC";
pub const MULTI_DOC4_INPUT: &str = r"
---
# Empty
...
%YAML 1.2
---";
pub const MULTI_DOC4_EVENTS: &str = r"
+DOC
=VAL :
-DOC
+DOC
=VAL :
-DOC";
pub const DOC_MAP_ERR_INPUT: &str = r"
--- a: b";
pub const DOC_MAP_ERR_EVENTS: &str = r"
+DOC
ERR
+MAP
=VAL :a
=VAL :b
-MAP
-DOC";
pub const X1_3HFZ_INPUT: &str = r"
---
a: b
... invalid";
pub const X1_3HFZ_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL :b
-MAP
-DOC
ERR
+DOC
=VAL :invalid
-DOC";
pub const X1_9HCY_INPUT: &str = r#"
!foo "bar"
%TAG ! tag:example.com,2000:app/
---
!foo "bar""#;
pub const X1_9HCY_EVENTS: &str = r#"
+DOC
=VAL <!foo> "bar
-DOC
ERR
+DOC
=VAL <tag:example.com,2000:app/foo> "bar
-DOC"#;
pub const X1_EB22_INPUT: &str = r#"
---
scalar1 # comment
%YAML 1.2"#;
pub const X1_EB22_EVENTS: &str = r#"
+DOC
=VAL :scalar1
-DOC
ERR
%YAML 1.2
ERR"#;
pub const DQUOTE_STR_ESC1_INPUT: &str = r#"
 "double quote (\")""#;
pub const DQUOTE_STR_ESC_EVENTS: &str = r#"
+DOC
=VAL "double quote (")
-DOC"#;
pub const DQUOTE_ESC1_INPUT: &str = r#"
 "a\/b"
"#;
pub const DQUOTE_ESC1_EVENTS: &str = r#"
+DOC
=VAL "a/b
-DOC"#;
pub const DQUOTE_ESC2_INPUT: &str = r#"
"foo\nbar\\baz": 23"#;
pub const DQUOTE_ESC2_EVENTS: &str = r#"
+DOC
+MAP
=VAL "foo\nbar\\baz
=VAL :23
-MAP
-DOC"#;
pub const X1_NP9H_INPUT: &str = r#"
"folded
to a space,

to a line feed, or 	\
 \ 	non-content""#;
pub const X1_NP9H_EVENTS: &str = r#"
+DOC
=VAL "folded to a space,\nto a line feed, or \t \tnon-content
-DOC"#;

pub const SQUOTE_STR1_INPUT: &str = r"
  'single quote'
    ";
pub const SQUOTE_STR2_INPUT: &str = r"
  'single
  quote'";
pub const SQUOTE_STR_EVENTS: &str = r"
+DOC
=VAL 'single quote
-DOC";
pub const SQUOTE_ESCAPE_INPUT: &str = r"'for single quote, use '' two of them'";
pub const SQUOTE_ESCAPE2_INPUT: &str = r"'for single quote, use
'' two of them'";
pub const SQUOTE_ESCAPE_EVENTS: &str = r"
+DOC
=VAL 'for single quote, use ' two of them
-DOC";
pub const DQUOTE_STR1_INPUT: &str = r#"
  "double quote"
    "#;
pub const DQUOTE_STR2_INPUT: &str = r#"
  "double
  quote"
"#;
pub const DQUOTE_STR_EVENTS: &str = r#"
+DOC
=VAL "double quote
-DOC"#;
pub const DQUOTE_MULTI_INPUT: &str = r##"
 "test

   tab" "##;
pub const DQUOTE_MULTI_EVENTS: &str = r#"
+DOC
=VAL "test\ntab
-DOC"#;
pub const DQUOTE_MULTI1_INPUT: &str = r#"
  gen: "\
      foo\
      bar   
      baz "
"#;
pub const DQUOTE_MULTI1_EVENTS: &str = r#"
+DOC
+MAP
=VAL :gen
=VAL "foobar baz 
-MAP
-DOC"#;
pub const DQUOTE_MULTI2_INPUT: &str = r##"
 - "double   
             
 quote" "##;
pub const DQUOTE_MULTI2_EVENTS: &str = r#"
+DOC
+SEQ
=VAL "double\nquote
-SEQ
-DOC"#;
pub const X_6WPF_INPUT: &str = r#"
"
  baz
""#;
pub const X_6WPF_EVENTS: &str = r#"
+DOC
=VAL " baz 
-DOC"#;
pub const DQUOTE_END_INPUT: &str = r#"
"
---
""#;
pub const DQUOTE_END_EVENTS: &str = r#"
+DOC
ERR"#;
pub const DQUOTE_ERR2_INPUT: &str = r#"
"\c"
"#;
pub const DQUOTE_ERR2_EVENTS: &str = r#"
+DOC
ERR"#;
pub const DQUOTE_MISS_EOF_INPUT: &str = r#"
---
key: "missing

"#;
pub const DQUOTE_MISS_EOF_EVENTS: &str = r#"
+DOC
+MAP
=VAL :key
ERR"#;
pub const DQUOTE_INDENT_ERR_INPUT: &str = r#"
---
quoted: "a
b
c"

"#;
pub const DQUOTE_INDENT_ERR_EVENTS: &str = r#"
+DOC
+MAP
=VAL :quoted
ERR"#;
pub const DQUOTE_COMMENT_ERR_INPUT: &str = r##"
---
"quote"# invalid comment

"##;
pub const DQUOTE_COMMENT_ERR_EVENTS: &str = r#"
+DOC
=VAL "quote
ERR"#;
pub const DQUOTE_LEADING_TAB1_INPUT: &str = r#" "1 test
    \	tab" "#;
pub const DQUOTE_LEADING_TAB2_INPUT: &str = r#"
    "1 test
      \ttab" "#;
pub const DQUOTE_LEADING_TAB3_INPUT: &str = r#"
"1 test\t
    tab" "#;
pub const DQUOTE_LEADING_TAB4_INPUT: &str = r#"
    "1 test\t
        tab" "#;
pub const DQUOTE_LEADING_TAB5_INPUT: &str = r#"
    "1 test\	
        tab"   "#;
pub const DQUOTE_LEADING_TAB_EVENTS: &str = r#"
+DOC
=VAL "1 test \ttab
-DOC"#;
pub const DQUOTE_LEADING_TAB2_EVENTS: &str = r#"
+DOC
=VAL "1 test\t tab
-DOC"#;

pub const DQUOTE_EMPTY1_INPUT: &str = r"
a: '
  '
b: '  
  '
  ";
pub const DQUOTE_EMPTY1_EVENTS: &str = r"
+DOC
+MAP
=VAL :a
=VAL ' 
=VAL :b
=VAL ' 
-MAP
-DOC";

pub const X1_G4RS_INPUT: &str = r#"unicode: "Sosa did fine.\u263A""#;
pub const X2_G4RS_INPUT: &str = r#"unicode: "Sosa did fine.☺""#;
pub const X1_G4RS_EVENTS: &str = r#"
+DOC
+MAP
=VAL :unicode
=VAL "Sosa did fine.☺
-MAP
-DOC"#;
pub const X3_G4RS_INPUT: &str = r#"hex esc: "\x0d\x0a is \r\n""#;
pub const X3_G4RS_EVENTS: &str = r#"
+DOC
+MAP
=VAL :hex esc
=VAL "\r\n is \r\n
-MAP
-DOC"#;
pub const NULL_YAML_INPUT: &str = r"
null
";
pub const NULL_YAML2_INPUT: &str = "\r\nnull\r\n";
pub const NULL_YAML_EVENTS: &str = r"
+DOC
=VAL :null
-DOC";
pub const MULTI_WORD_INPUT: &str = r"
  null test xy";
pub const MULTI_WORD_EVENTS: &str = r"
+DOC
=VAL :null test xy
-DOC";
pub const MULTILINE_INPUT: &str = r"
test
xt
";
pub const MULTILINE_EVENTS: &str = r"
+DOC
=VAL :test xt
-DOC";
pub const SEQ_FLOW_INPUT: &str = r"
[x, y]
";
pub const SEQ_FLOW2_INPUT: &str = r"
[x ,y]
";
pub const SEQ_FLOW_EVENTS: &str = r"
+DOC
+SEQ
=VAL :x
=VAL :y
-SEQ
-DOC";
pub const NEST_COL1_INPUT: &str = r"
[:]
";
pub const NEST_COL2_INPUT: &str = r"
[{:}]
";
pub const NESTED_COL_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL :
=VAL :
-MAP
-SEQ
-DOC";
pub const MAP_XY_INPUT: &str = r"
{x:y}
";
pub const MAP_XY_EVENTS: &str = r"
+DOC
+MAP
=VAL :x:y
=VAL :
-MAP
-DOC";
pub const MAP_X_Y_INPUT: &str = r"
{x: y}
";
pub const MAP_X_Y2_INPUT: &str = r"
{? x: y}
";
pub const MAP_X_Y3_INPUT: &str = r"
{x: #comment
 y}
";
pub const MAP_X_Y_EVENTS: &str = r"
+DOC
+MAP
=VAL :x
=VAL :y
-MAP
-DOC";
pub const COMPLEX_MAP_INPUT: &str = r"
{[x,y]:a}
";
pub const COMPLEX_MAP_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :x
=VAL :y
-SEQ
=VAL :a
-MAP
-DOC";
pub const X1_9MMW_INPUT: &str = r#"
[ "JSON like":adjacent ]"#;
pub const X1_9MMW_EVENTS: &str = r#"
+DOC
+SEQ
+MAP
=VAL "JSON like
=VAL :adjacent
-MAP
-SEQ
-DOC"#;
pub const X2_9MMW_INPUT: &str = r#"
[ {JSON: like}:adjacent ]"#;
pub const X2_9MMW_EVENTS: &str = r"
+DOC
+SEQ
+MAP
+MAP
=VAL :JSON
=VAL :like
-MAP
=VAL :adjacent
-MAP
-SEQ
-DOC";
pub const X1_WZ62_INPUT: &str = r"
{
    foo : !!str,
    !!str : bar,
  }";
pub const X1_WZ62_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL <tag:yaml.org,2002:str> :
=VAL <tag:yaml.org,2002:str> :
=VAL :bar
-MAP
-DOC";
pub const X1_1_ZXT5_INPUT: &str = r#"
[ "key"
  :value ]"#;
pub const X1_2_ZXT5_INPUT: &str = r#"
  [ "key"
    : value ]"#;
pub const X1_ZXT5_EVENTS: &str = r#"
+DOC
+SEQ
=VAL "key
ERR"#;
pub const FLOW_QUOTED1_INPUT: &str = r#"
{"ab"
: "xy"}
"#;
pub const FLOW_QUOTED2_INPUT: &str = r#"
{"ab"
:xy}
"#;
pub const FLOW_QUOTED1_EVENTS: &str = r#"
+DOC
+MAP
=VAL "ab
=VAL "xy
-MAP
-DOC"#;
pub const FLOW_QUOTED2_EVENTS: &str = r#"
+DOC
+MAP
=VAL "ab
=VAL :xy
-MAP
-DOC"#;
pub const X_C2DT_INPUT: &str = r#"
{
"empty":
} "#;
pub const X_C2DT_EVENTS: &str = r#"
+DOC
+MAP
=VAL "empty
=VAL :
-MAP
-DOC"#;
pub const EMPTY_MAP1_INPUT: &str = r"
{:}
";
pub const EMPTY_MAP2_INPUT: &str = r"
{ : }
";
pub const EMPTY_FLOW_MAP_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :
-MAP
-DOC";
pub const EMPTY_NODES_INPUT: &str = r#"
{
    a: "b",
    x,
    y:,
}
"#;
pub const EMPTY_NODES_EVENTS: &str = r#"
+DOC
+MAP
=VAL :a
=VAL "b
=VAL :x
=VAL :
=VAL :y
=VAL :
-MAP
-DOC"#;
pub const TWO_EMPTY_INPUT: &str = r"
{:, :}
";
pub const TWO_EMPTY_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :
=VAL :
=VAL :
-MAP
-DOC";
pub const ERR_PLAIN_SCALAR_INPUT: &str = r"
  a
  b
 c";
pub const ERR_PLAIN_SCALAR_EVENTS: &str = r"
+DOC
=VAL :a b c
-DOC";
pub const FLOW_ERR1_INPUT: &str = r"
---
[a, b] ]";
pub const FLOW_ERR1_EVENTS: &str = r"
+DOC
+SEQ
=VAL :a
=VAL :b
-SEQ
-DOC
ERR";
pub const FLOW_ERR2_INPUT: &str = r"
---
[ [a, b] ";
pub const FLOW_ERR2_EVENTS: &str = r"
+DOC
+SEQ
+SEQ
=VAL :a
=VAL :b
-SEQ
ERR";
pub const SEQ_ERR_INPUT: &str = r"
 [-]";
pub const SEQ_ERR_EVENTS: &str = r"
+DOC
+SEQ
ERR";
pub const X_9JBA_INPUT: &str = r"
 [a, b]#invalid";
pub const X_9JBA_EVENTS: &str = r"
+DOC
+SEQ
=VAL :a
=VAL :b
-SEQ
ERR";
pub const X_9MAG_INPUT: &str = r"
[ , a , b, c] ";
pub const X_9MAG_EVENTS: &str = r"
+DOC
+SEQ
ERR";
pub const X_CML9_INPUT: &str = r"
key: [ word1
  #  xxx
  word2 ]";
pub const X_CML9_EVENTS: &str = r"
+DOC
+MAP
=VAL :key
+SEQ
=VAL :word1
ERR";
pub const X1_CVW2_INPUT: &str = r"
[a,#comment
]";
pub const X1_CVW2_EVENTS: &str = r"
+DOC
+SEQ
=VAL :a
ERR";
pub const X2_CVW2_INPUT: &str = r"
[a, #comment
]";
pub const X2_CVW2_EVENTS: &str = r"
+DOC
+SEQ
=VAL :a
-SEQ
-DOC";
pub const X1_N782_INPUT: &str = r"
[
---
]";
pub const X1_N782_EVENTS: &str = r"
+DOC
+SEQ
ERR";
pub const X2_N782_INPUT: &str = r"
{
---
}";
pub const X2_N782_EVENTS: &str = r"
+DOC
+MAP
ERR";
pub const SEQ_KEY1_INPUT: &str = r"
[a, b]: 3 ";
pub const SEQ_KEY1_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :a
=VAL :b
-SEQ
=VAL :3
-MAP
-DOC";
pub const SEQ_KEY2_INPUT: &str = r"
[a, [b,c]]: 3 ";
pub const SEQ_KEY2_EVENTS: &str = r"
+DOC
+MAP
+SEQ
=VAL :a
+SEQ
=VAL :b
=VAL :c
-SEQ
-SEQ
=VAL :3
-MAP
-DOC";
pub const SEQ_KEY3_INPUT: &str = r"
 [[a]: 3]";
pub const SEQ_KEY3_EVENTS: &str = r"
+DOC
+SEQ
+MAP
+SEQ
=VAL :a
-SEQ
=VAL :3
-MAP
-SEQ
-DOC";
pub const SEQ_KEY4_INPUT: &str = r"
 [ [a]: d, e]: 3";
pub const SEQ_KEY4_EVENTS: &str = r"
+DOC
+MAP
+SEQ
+MAP
+SEQ
=VAL :a
-SEQ
=VAL :d
-MAP
=VAL :e
-SEQ
=VAL :3
-MAP
-DOC";
pub const SEQ_EDGE_INPUT: &str = r"
 [:x]";
pub const SEQ_EDGE_EVENTS: &str = r"
+DOC
+SEQ
=VAL ::x
-SEQ
-DOC";
pub const X1_8UDB_INPUT: &str = r"
[
single: pair,
]";
pub const X1_8UDB_EVENTS: &str = r#"
+DOC
+SEQ
+MAP
=VAL :single
=VAL :pair
-MAP
-SEQ
-DOC"#;
pub const X2_8UDB_INPUT: &str = r"
[[ ],
single: pair,]";
pub const X2_8UDB_EVENTS: &str = r"
+DOC
+SEQ
+SEQ
-SEQ
+MAP
=VAL :single
=VAL :pair
-MAP
-SEQ
-DOC";
pub const MAP_EDGE1_INPUT: &str = r"
 {x: :x}";
pub const MAP_EDGE1_EVENTS: &str = r"
+DOC
+MAP
=VAL :x
=VAL ::x
-MAP
-DOC";
pub const MAP_EDGE2_INPUT: &str = r"
 {:x}";
pub const MAP_EDGE2_EVENTS: &str = r"
+DOC
+MAP
=VAL ::x
=VAL :
-MAP
-DOC";
pub const MAP_ERR_INPUT: &str = r"
[23
]: 42";
pub const MAP_ERR_EVENTS: &str = r"
+DOC
+SEQ
=VAL :23
ERR";
pub const X_CT4Q_INPUT: &str = r"
[? foo
    bar: baz ]";
pub const X_CT4Q_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL :foo bar
=VAL :baz
-MAP
-SEQ
-DOC";
pub const X_DFF7_INPUT: &str = r"
{
?
}";
pub const X_DFF7_EVENTS: &str = r"
+DOC
+MAP
=VAL :
=VAL :
-MAP
-DOC";
pub const X1_DK4H_INPUT: &str = r"
[ key
  : value]";
pub const X1_DK4H_EVENTS: &str = r"
+DOC
+SEQ
=VAL :key
ERR";
pub const X2_DK4H_INPUT: &str = r"
[ ? key
  : value]";
pub const X2_DK4H_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL :key
=VAL :value
-MAP
-SEQ
-DOC";
pub const X1_T833_INPUT: &str = r"
{
    foo: 1
    bar: 2
}";
pub const X1_T833_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
ERR";
pub const FLOW_TAG_INPUT: &str = r"
%TAG !m! !my-
--- # Bulb here
!m!light fluorescent
...";
pub const FLOW_TAG_EVENTS: &str = r"
+DOC
=VAL <!my-light> :fluorescent
-DOC";
pub const X1_EHF6_INPUT: &str = r"
!!map {
    k: !!seq [a, !!str b]
}";
pub const X1_EHF6_EVENTS: &str = r"
+DOC
+MAP <tag:yaml.org,2002:map>
=VAL :k
+SEQ <tag:yaml.org,2002:seq>
=VAL :a
=VAL <tag:yaml.org,2002:str> :b
-SEQ
-MAP
-DOC";
pub const X1_CN3R_INPUT: &str = r"
[
 { &e e: f },
 &g { g: h }
]";
pub const X1_CN3R_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL &e :e
=VAL :f
-MAP
+MAP &g
=VAL :g
=VAL :h
-MAP
-SEQ
-DOC";
pub const X2_CN3R_INPUT: &str = r"
  { &e e: f }
";
pub const X2_CN3R_EVENTS: &str = r"
+DOC
+MAP
=VAL &e :e
=VAL :f
-MAP
-DOC";
pub const X3_CN3R_INPUT: &str = r"
[&c c: d]
";
pub const X3_CN3R_EVENTS: &str = r"
+DOC
+SEQ
+MAP
=VAL &c :c
=VAL :d
-MAP
-SEQ
-DOC";
pub const X4_CN3R_INPUT: &str = r"
[&g {g: h}]";
pub const X4_CN3R_EVENTS: &str = r"
+DOC
+SEQ
+MAP &g
=VAL :g
=VAL :h
-MAP
-SEQ
-DOC";
pub const FLOW_ALIAS_INPUT: &str = r"
&seq [ &item 'a']
";
pub const FLOW_ALIAS_EVENTS: &str = r"
+DOC
+SEQ &seq
=VAL &item 'a
-SEQ
-DOC";
pub const X1_X38W_INPUT: &str = r"
{&a []: *a}
";
pub const X1_X38W_EVENTS: &str = r"
+DOC
+MAP
+SEQ &a
-SEQ
=ALI *a
-MAP
-DOC";
pub const X1_Y79Y_003_INPUT: &str = r"
- [
 	foo,
 foo,
	 foo,
 ]";
pub const X1_Y79Y_003_EVENTS: &str = r"
+DOC
+SEQ
+SEQ
=VAL :foo
=VAL :foo
ERR";
pub const X1_5T43_INPUT: &str = r#"
- { "key":value }
- { "key"::value }"#;
pub const X1_5T43_EVENTS: &str = r#"
+DOC
+SEQ
+MAP
=VAL "key
=VAL :value
-MAP
+MAP
=VAL "key
=VAL ::value
-MAP
-SEQ
-DOC"#;
pub const X1_FRK4_INPUT: &str = r"
{
    ? foo :,
    : bar,
}";
pub const X1_FRK4_EVENTS: &str = r"
+DOC
+MAP
=VAL :foo
=VAL :
=VAL :
=VAL :bar
-MAP
-DOC";
