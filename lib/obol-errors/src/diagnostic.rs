//! A universal diagnostic engine for error.
//! 
//! This module defines the core components of the diagnostic system, including:
//! 
//! - `DiagnosticId` : A unique identifier for each diagnostic, represented as a `u32`.
//! - `RawDiagnostic` : A raw diagnostic structure containing the diagnostic ID and an optional location
//! - `Diagnostic` : A structured diagnostic that includes a `RawDiagnostic` and a phantom type for context.

pub trait AsDiagnosticId {
    type Id: Sized + Copy;
    const ID: Self::Id;
    const U32_ID: u32;
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub(crate) struct DiagnosticId(u32);

impl DiagnosticId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    pub const fn as_u32(&self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for DiagnosticId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

impl core::fmt::Debug for DiagnosticId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct RawDiagnostic {
    pub(crate) id: DiagnosticId,
    pub(crate) location: &'static std::panic::Location<'static>,
}

impl std::fmt::Display for RawDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, location: {:?}", self.id, self.location)
    }
}

impl std::fmt::Debug for RawDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "id: {}, location: {:?}", self.id, self.location)
    }
}

pub struct Diagnostic<'a, T> {
    pub inner: RawDiagnostic,
    _marker: core::marker::PhantomData<&'a T>,
}

impl<'a, T> Diagnostic<'a, T>
where
    T: AsDiagnosticId,
    [(); const { (T::U32_ID < 0xFFFF) as usize }]:,
{
    pub const fn new(self) -> Self {
        Self {
            inner: RawDiagnostic {
                id: DiagnosticId::new(DiagnosticId::new(T::U32_ID).as_u32()),
                location: &std::panic::Location::caller(),
            },
            _marker: core::marker::PhantomData,
        }
    }
}
