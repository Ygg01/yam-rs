use crate::node::Yaml;
use crate::prelude::{ToMut, ToMutStr, YamlDocAccess};
use alloc::string::{String, ToString};

/// Macro for fast implementation generation
/// Ensures all types have easy to access conversion.
///
/// # Params:
/// - `$ty` - Ident of type being compared
/// - `$as_ty` - Ident (name) of a method from `YamDocAccess` that provides access to underlying type
/// - `$from`- Optional. If present the macro will use `$from::from(other)` conversion.
///   otherwise it doesn't use any conversion.
macro_rules! partialeq_numeric {
    ($($as_method:ident $num:ty [$($ty:ty)*])*) => {
        $($(
            impl<'d, F, I, S>  PartialEq<$ty> for Yaml<'d, F, I, S>
            where
                F: Copy + ToMut<f64> + PartialEq,
                I: Copy + ToMut<i64> + PartialEq,
                S: Clone + ToMutStr<'d> + PartialEq,
            {
                fn eq(&self, other: &$ty) -> bool {
                    self.$as_method() == Some(*other as $num)
                }
            }

            impl<'d, F, I, S>  PartialEq<Yaml<'d, F, I, S> > for $ty
            where
                F: Copy + ToMut<f64> + PartialEq,
                I: Copy + ToMut<i64> + PartialEq,
                S: Clone + ToMutStr<'d> + PartialEq,
            {
                fn eq(&self, other: &Yaml<'d, F, I, S>) -> bool {
                    other.$as_method() == Some(*self as $num)
                }
            }

            impl<'d, F, I, S>  PartialEq<&$ty > for Yaml<'d, F, I, S>
            where
                F: Copy + ToMut<f64> + PartialEq,
                I: Copy + ToMut<i64> + PartialEq,
                S: Clone + ToMutStr<'d> + PartialEq,
            {
                fn eq(&self, other: &&$ty) -> bool {
                    self.$as_method() == Some(**other as $num)
                }
            }

            impl<'d, F, I, S> PartialEq<$ty> for &Yaml<'d, F, I, S>
            where
                F: Copy + ToMut<f64> + PartialEq,
                I: Copy + ToMut<i64> + PartialEq,
                S: Clone + ToMutStr<'d> + PartialEq,
            {
                fn eq(&self, other: &$ty) -> bool {
                    self.$as_method() == Some(*other as $num)
                }
            }

            impl<'a, 'd, F, I, S > PartialEq<$ty> for &mut Yaml<'d, F, I, S>
            where
                F: Copy + ToMut<f64> + PartialEq,
                I: Copy + ToMut<i64> + PartialEq,
                S: Clone + ToMutStr<'d> + PartialEq,
            {    fn eq(&self, other: &$ty) -> bool {
                    self.$as_method() == Some(*other as $num)
                }
            }
        )*)*
    }
}

partialeq_numeric!(
    as_i64 i64 [i8 i16 i32 i64 isize]
    as_i64 i64 [u8 u16 u32]
    as_f64 f64 [f32 f64]
    as_bool bool [bool]
);

/// This conversion is fallible, numbers bigger than i64 can't be stored
/// in `Yaml`
impl<'a, F, I, S> PartialEq<u64> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    fn eq(&self, other: &u64) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for u64
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_i64().eq(&i64::try_from(*self).ok())
    }
}

impl<'a, F, I, S> PartialEq<u64> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    fn eq(&self, other: &u64) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

impl<'a, F, I, S> PartialEq<u64> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    fn eq(&self, other: &u64) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

/// This conversion is potentially fallible on 64-bit systems,
/// numbers bigger than i64 can't be stored
/// in `Yaml`
impl<'a, F, I, S> PartialEq<usize> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    fn eq(&self, other: &usize) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for usize
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_i64().eq(&i64::try_from(*self).ok())
    }
}

impl<'a, F, I, S> PartialEq<usize> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    fn eq(&self, other: &usize) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

impl<'a, F, I, S> PartialEq<usize> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    fn eq(&self, other: &usize) -> bool {
        self.as_i64().eq(&i64::try_from(*other).ok())
    }
}

/// str conversions are a bit more tricky
impl<'a, F, I, S> PartialEq<&str> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for &str
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_str().eq(&Some(*self))
    }
}

impl<'a, F, I, S> PartialEq<&str> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<&str> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str().eq(&Some(other))
    }
}

/// String conversions are a bit more tricky
impl<'a, F, I, S> PartialEq<String> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for String
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_str().eq(&Some(self))
    }
}

impl<'a, F, I, S> PartialEq<&String> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &&String) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<String> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(&Some(other))
    }
}

impl<'a, F, I, S> PartialEq<String> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str().eq(&Some(other))
    }
}

/// Char conversion is a bit more involved than others.
impl<'a, F, I, S> PartialEq<char> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.as_str().eq(&Some(&other.to_string()))
    }
}

impl<'a, F, I, S> PartialEq<&char> for Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &&char) -> bool {
        self.as_str().eq(&Some(&other.to_string()))
    }
}

impl<'a, F, I, S> PartialEq<Yaml<'a, F, I, S>> for char
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Yaml<'a, F, I, S>) -> bool {
        other.as_str().eq(&Some(&self.to_string()))
    }
}

impl<'a, F, I, S> PartialEq<char> for &Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.as_str().eq(&Some(&other.to_string()))
    }
}

impl<'a, F, I, S> PartialEq<char> for &mut Yaml<'a, F, I, S>
where
    F: Copy + ToMut<f64> + PartialEq,
    I: Copy + ToMut<i64> + PartialEq,
    S: Clone + ToMutStr<'a> + PartialEq,
{
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.as_str().eq(&Some(&other.to_string()))
    }
}
