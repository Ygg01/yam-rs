#[cfg(target_arch = "aarch64")]
mod aarch64;
pub(crate) mod avx2;
pub(crate) mod native;
pub(crate) mod sse42;
