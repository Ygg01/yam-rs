//! Module containing platform specific implementations
pub use avx_scanner::AvxScanner;
pub use native_scanner::NativeScanner;

// mod avx_stage1;
mod avx_scanner;
mod native_scanner;
