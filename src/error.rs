//! Error handling.
//!
//! using feature gat v2
//!

pub type Result<T, E = ErrorReport> = core::result::Result<T, E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Validation,
    Trade,
    Database,
    Internal,
}

impl core::fmt::Display for ErrorKind {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

#[derive(Debug)]
pub struct ErrorReport {
    pub code: u32,
    pub kind: ErrorKind,
    pub message: String,
    pub source: Option<Box<dyn core::error::Error + Send + Sync>>,
}

impl core::error::Error for ErrorReport {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }

    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        request.provide_value(self.code);
        request.provide_value(self.kind);
    }
}

impl core::fmt::Display for ErrorReport {
    fn fmt(
        &self,
        fmt: &mut core::fmt::Formatter<'_>,
    ) -> core::result::Result<(), core::fmt::Error> {
        write!(
            fmt,
            "[ERROR:{:#06X}] {} -> {}",
            self.code, self.kind, self.message
        )?;
        if let Some(ref src) = self.source {
            write!(fmt, " (Cause by: {})", src)?;
        }
        Ok(())
    }
}

/// Est-ce que utilise gat v2 ici pourrais etre interessant
pub trait ErrorContextExt<T> {
    type Out<'a> where Self: 'a;

    fn with_lazy_context<F, S>(self, f: F) -> Self::Out<'static>
    where
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>;
}

impl<T, E> ErrorContextExt<T> for core::result::Result<T, E>
where
    E: core::error::Error + Send + Sync + 'static,
{
    type Out<'a> = Result<T, ErrorReport> where Self: 'a;

    fn with_lazy_context<F, S>(self, f: F) -> Self::Out<'static>
    where
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>,
    {
        self.map_err(|e| {
            let (kind, code, msg) = f();
            ErrorReport {
                code,
                kind,
                message: msg.into(),
                source: Some(Box::new(e)),
            }
        })
    }
}

// Implémentation pour Option
impl<T> ErrorContextExt<T> for Option<T> {
    type Out<'a> = Result<T, ErrorReport> where Self: 'a;

    fn with_lazy_context<F, S>(self, f: F) -> Self::Out<'static>
    where
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>,
    {
        self.ok_or_else(|| {
            let (kind, code, msg) = f();
            ErrorReport {
                code,
                kind,
                message: msg.into(),
                source: None,
            }
        })
    }
}