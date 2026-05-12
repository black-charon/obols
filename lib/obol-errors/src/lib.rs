//! # Obol-Errors ::
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(adt_const_params)]
#![feature(min_generic_const_args)]
#![feature(associated_type_defaults)]
#![feature(const_convert)]

pub mod context;
pub mod diagnostic;
pub mod macros;
pub mod report;

///
pub type Result<T, E> = core::result::Result<T, E>;
