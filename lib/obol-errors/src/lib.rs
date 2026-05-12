//! # Obol-Errors ::
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(const_trait_impl)]
#![feature(adt_const_params)]
#![feature(min_generic_const_args)]

pub mod context;
pub mod diagnostic;
pub mod macros;
pub mod report;

///
pub type Result<T, E = report::ErrorReport> = core::result::Result<T, E>;

pub mod prelude {
    pub use crate::Result;
    pub use crate::context::{ContextValue, ErrorContext, ErrorContextExt};
    pub use crate::diagnostic::{AsDiagnosticId, Diagnostic, RawDiagnostic};
    pub use crate::new_error;
    pub use crate::report::ErrorReport;
}
