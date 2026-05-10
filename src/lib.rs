//! A Standard Library for Black Charon
//!
#![no_std]
#![feature(portable_simd)]
#![feature(const_trait_impl)]
#![feature(maybe_uninit_slice)]
#![feature(core_intrinsics)]

mod error;
mod string;

pub mod prelude {
    pub use crate::string::String;
}

pub mod internal {
    pub trait Sealed {}
}
