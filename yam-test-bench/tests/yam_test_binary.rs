use rstest::rstest;
use yam_core::binary::{decode_as_base64, encode_as_base64};

const STR1_INPUT: &[u8] = b"This is complete shite!".as_slice();
const STR2_INPUT: &[u8] = b"This is complete shite!!".as_slice();
const STR3_INPUT: &[u8] = b"This is complete shite!!!".as_slice();
const STR1_B64: &str = "VGhpcyBpcyBjb21wbGV0ZSBzaGl0ZSE=";
const STR2_B64: &str = "VGhpcyBpcyBjb21wbGV0ZSBzaGl0ZSEh";
const STR3_B64: &str = "VGhpcyBpcyBjb21wbGV0ZSBzaGl0ZSEhIQ==";

#[rstest]
#[case::str1(STR1_INPUT, STR1_B64)]
#[case::str2(STR2_INPUT, STR2_B64)]
#[case::str3(STR3_INPUT, STR3_B64)]
fn test_encode(#[case] input: &[u8], #[case] expected: &str) {
    let enc = encode_as_base64(input);
    assert_eq!(enc, expected);
}

#[rstest]
#[case::str1(STR1_B64, STR1_INPUT)]
#[case::str2(STR2_B64, STR2_INPUT)]
#[case::str3(STR3_B64, STR3_INPUT)]
fn test_decode(#[case] input: &str, #[case] expected: &[u8]) {
    let decode = decode_as_base64(input);
    assert!(
        decode.is_ok(),
        "Expected decode to be successful got {} instead",
        decode.err().unwrap()
    );
    assert_eq!(decode.unwrap().as_slice(), expected);
}
