mod u8x16;
mod u8x32;

pub(crate) use u8x16::U8X16;
pub(crate) use u8x32::U8X32;

#[doc(hidden)]
pub(crate) const LOW_NIBBLE_WS: U8X16 =
    U8X16::from_array([4, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 2, 0, 0]);
#[doc(hidden)]
pub(crate) const HIGH_NIBBLE_WS: U8X16 =
    U8X16::from_array([3, 0, 12, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);

pub trait BitOps {
    type ByteOut;

    fn comp_to_bitmask(self, cmp: u8) -> Self::ByteOut;

    fn comp(self, cmp: u8) -> Self;
    fn to_bitmask(&self) -> Self::ByteOut;

    fn swizzle(self, other: Self) -> Self;

    fn and(self, other: Self) -> Self;

    fn and_byte(self, other: u8) -> Self;

    fn shift_right(self, rhs: usize) -> Self;
    fn add_other(self, other: Self) -> Self;
}

#[macro_use]
mod macros {
    macro_rules! gen_u8_cmp {
        ($left:expr , $n:literal => $cmp:expr) => {
            (if $left[$n] == $cmp { 1 << $n } else { 0 })
        };
    }

    macro_rules! gen_u8_cmp_all {
        ($left:expr , $n:literal => $cmp:expr) => {
            if $left[$n] == $cmp { 0xFF } else { 0x00 }
        };
    }

    macro_rules! swizzle {
        ($mask:expr, $n:literal => $right:expr) => {
            if $right[$n] > 0x0f {
                0
            } else {
                $mask[($right[$n] & 0x0f) as usize]
            }
        };
    }

    macro_rules! bitmask {
        ($mask:expr, $n:literal => $out:ident) => {
            ($out::from($mask[$n] & 0b1000_0000 != 0)) << $n
        };
    }

    pub(crate) use bitmask;
    pub(crate) use gen_u8_cmp;
    pub(crate) use gen_u8_cmp_all;
    pub(crate) use swizzle;
}
