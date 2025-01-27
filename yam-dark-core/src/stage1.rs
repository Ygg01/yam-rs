use std::mem::transmute;
use simdutf8::basic::imp::ChunkedUtf8Validator;

pub(crate) trait Stage1Parse {
    type Utf8Validator: ChunkedUtf8Validator;
    type SimdRepresentation;

    unsafe fn new(ptr: &[u8]) -> Self;

    unsafe fn compute_quote_mask(quote_bits: u64) -> u64;

    unsafe fn cmp_mask_against_input(&self, m: u8) -> u64;

    unsafe fn unsigned_lteq_against_input(&self, maxval: Self::SimdRepresentation) -> u64;

    unsafe fn find_whitespace_and_structurals(&self, whitespace: &mut u64, structurals: &mut u64);

    unsafe fn flatten_bits(base: &mut Vec<u32>, idx: u32, bits: u64);

    unsafe fn fill_s8(n: i8) -> Self::SimdRepresentation;

    unsafe fn zero() -> Self::SimdRepresentation;


    // return both the quote mask (which is a half-open mask that covers the first
    // quote in an unescaped quote pair and everything in the quote pair) and the
    // quote bits, which are the simple unescaped quoted bits.
    //
    // We also update the prev_iter_inside_quote value to tell the next iteration
    // whether we finished the final iteration inside a quote pair; if so, this
    // inverts our behavior of whether we're inside quotes for the next iteration.
    //
    // Note that we don't do any error checking to see if we have backslash
    // sequences outside quotes; these
    // backslash sequences (of any length) will be detected elsewhere.
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn find_quote_mask_and_bits(
        &self,
        odd_ends: u64,
        prev_iter_inside_quote: &mut u64,
        quote_bits: &mut u64,
        error_mask: &mut u64,
    ) -> u64 {
        unsafe {
            *quote_bits = self.cmp_mask_against_input(b'"');
            *quote_bits &= !odd_ends;
            // remove from the valid quoted region the unescaped characters.
            let mut quote_mask: u64 = Self::compute_quote_mask(*quote_bits);
            quote_mask ^= *prev_iter_inside_quote;
            // All Unicode characters may be placed within the
            // quotation marks, except for the characters that MUST be escaped:
            // quotation mark, reverse solidus, and the control characters (U+0000
            //through U+001F).
            // https://tools.ietf.org/html/rfc8259
            let unescaped: u64 = self.unsigned_lteq_against_input(Self::fill_s8(0x1F));
            *error_mask |= quote_mask & unescaped;
            // right shift of a signed value expected to be well-defined and standard
            // compliant as of C++20,
            // John Regher from Utah U. says this is fine code
            *prev_iter_inside_quote = transmute::<_, u64>(transmute::<_, i64>(quote_mask) >> 63);
            quote_mask
        }
    }

    // return a bitvector indicating where we have characters that end an odd-length
    // sequence of backslashes (and thus change the behavior of the next character
    // to follow). A even-length sequence of backslashes, and, for that matter, the
    // largest even-length prefix of our odd-length sequence of backslashes, simply
    // modify the behavior of the backslashes themselves.
    // We also update the prev_iter_ends_odd_backslash reference parameter to
    // indicate whether we end an iteration on an odd-length sequence of
    // backslashes, which modifies our subsequent search for odd-length
    // sequences of backslashes in an obvious way.
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn find_odd_backslash_sequences(&self, prev_iter_ends_odd_backslash: &mut u64) -> u64 {
        const EVEN_BITS: u64 = 0x5555_5555_5555_5555;
        const ODD_BITS: u64 = !EVEN_BITS;

        let bs_bits: u64 = unsafe { self.cmp_mask_against_input(b'\\') };
        let start_edges: u64 = bs_bits & !(bs_bits << 1);
        // flip lowest if we have an odd-length run at the end of the prior
        // iteration
        let even_start_mask: u64 = EVEN_BITS ^ *prev_iter_ends_odd_backslash;
        let even_starts: u64 = start_edges & even_start_mask;
        let odd_starts: u64 = start_edges & !even_start_mask;
        let even_carries: u64 = bs_bits.wrapping_add(even_starts);

        // must record the carry-out of our odd-carries out of bit 63; this
        // indicates whether the sense of any edge going to the next iteration
        // should be flipped
        let (mut odd_carries, iter_ends_odd_backslash) = bs_bits.overflowing_add(odd_starts);

        odd_carries |= *prev_iter_ends_odd_backslash;
        // push in bit zero as a potential end
        // if we had an odd-numbered run at the
        // end of the previous iteration
        *prev_iter_ends_odd_backslash = u64::from(iter_ends_odd_backslash);
        let even_carry_ends: u64 = even_carries & !bs_bits;
        let odd_carry_ends: u64 = odd_carries & !bs_bits;
        let even_start_odd_end: u64 = even_carry_ends & ODD_BITS;
        let odd_start_even_end: u64 = odd_carry_ends & EVEN_BITS;
        let odd_ends: u64 = even_start_odd_end | odd_start_even_end;
        odd_ends
    }

    // return a updated structural bit vector with quoted contents cleared out and
    // pseudo-structural characters added to the mask
    // updates prev_iter_ends_pseudo_pred which tells us whether the previous
    // iteration ended on a whitespace or a structural character (which means that
    // the next iteration
    // will have a pseudo-structural character at its start)
    #[cfg_attr(not(feature = "no-inline"), inline)]
    fn finalize_structurals(
        mut structurals: u64,
        whitespace: u64,
        quote_mask: u64,
        quote_bits: u64,
        prev_iter_ends_pseudo_pred: &mut u64,
    ) -> u64 {
        // mask off anything inside quotes
        structurals &= !quote_mask;
        // add the real quote bits back into our bitmask as well, so we can
        // quickly traverse the strings we've spent all this trouble gathering
        structurals |= quote_bits;
        // Now, establish "pseudo-structural characters". These are non-whitespace
        // characters that are (a) outside quotes and (b) have a predecessor that's
        // either whitespace or a structural character. This means that subsequent
        // passes will get a chance to encounter the first character of every string
        // of non-whitespace and, if we're parsing an atom like true/false/null or a
        // number we can stop at the first whitespace or structural character
        // following it.

        // a qualified predecessor is something that can happen 1 position before an
        // pseudo-structural character
        let pseudo_pred: u64 = structurals | whitespace;

        let shifted_pseudo_pred: u64 = (pseudo_pred << 1) | *prev_iter_ends_pseudo_pred;
        *prev_iter_ends_pseudo_pred = pseudo_pred >> 63;
        let pseudo_structurals: u64 = shifted_pseudo_pred & (!whitespace) & (!quote_mask);
        structurals |= pseudo_structurals;

        // now, we've used our close quotes all we need to. So let's switch them off
        // they will be off in the quote mask and on in quote bits.
        structurals &= !(quote_bits & !quote_mask);
        structurals
    }
}