//! # Obol :: The Shared Foundation for Black Charon Microservices
//!
//! **Obol** est la bibliothèque socle distribuée à travers l'ensemble des microservices
//! de l'écosystème e-commerce **Black Charon**. Elle garantit la cohérence technique,
//! la sûreté des types et l'uniformité des diagnostics à travers tout le réseau de services.
//!
//!
//! ## Modules de la Crateà
//!
//! *   [`error`] : Le moteur de diagnostic universel.
#![allow(incomplete_features)]
#![feature(error_generic_member_access)]
#![feature(lazy_type_alias)]

pub mod error;

pub use crate::error::{ErrorContextExt, ErrorDef, ErrorKind, ErrorReport, Result};

pub mod prelude {}
