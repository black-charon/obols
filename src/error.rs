use std::fmt;
use std::error::{Error, Request};
use std::panic::Location;

// --- REGISTRE DES ERREURS (L'UNIQUE SOURCE DE VÉRITÉ) ---

pub trait Diagnostic {
    fn code(&self) -> u32;
    fn kind(&self) -> ErrorKind;
}

macro_rules! register_errors {
    (
        $name:ident {
            $($variant:ident => ($code:expr, $kind:ident)),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant),*
        }

        impl $crate::error::Diagnostic for $name {
            fn code(&self) -> u32 {
                match self { $(Self::$variant => $code),* }
            }
            fn kind(&self) -> $crate::error::ErrorKind {
                match self { $(Self::$variant => $crate::error::ErrorKind::$kind),* }
            }
        }
    };
}

// Définition de tes domaines d'erreurs
register_errors! {
    ErrorDef {
        // Validation (0x1xxx)
        InvalidSku     => (0x1001, Validation),
        InvalidPrice   => (0x1002, Validation),
        // Trade (0x2xxx)
        StockEmpty     => (0x2001, Trade),
        PaymentFailed  => (0x2002, Trade),
        // Database (0x3xxx)
        WalCorrupted   => (0x3001, Database),
        // Système (0xFxxx)
        InternalError  => (0xF001, Internal),
    }
}

// --- TYPES DE BASE ---

pub type Result<T, E = ErrorReport> = core::result::Result<T, E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Validation,
    Trade,
    Database,
    Internal,
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

// --- LE RAPPORT D'ERREUR (REPORT) ---

#[derive(Debug)]
pub struct ErrorReport {
    pub code: u32,
    pub kind: ErrorKind,
    pub message: String,
    pub location: &'static Location<'static>,
    pub source: Option<Box<dyn Error + Send + Sync>>,
}

impl Error for ErrorReport {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source.as_ref().map(|e| e.as_ref() as _)
    }

    fn provide<'a>(&'a self, request: &mut Request<'a>) {
        request.provide_value(self.code);
        request.provide_value(self.kind);
        request.provide_value(self.location);
    }
}

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[ERR:{:#06X}] {} -> {}", self.code, self.kind, self.message)?;
        write!(f, " (at {}:{})", self.location.file(), self.location.line())?;
        if let Some(ref src) = self.source {
            write!(f, " | Source: {}", src)?;
        }
        Ok(())
    }
}

impl ErrorReport {
    #[track_caller]
    #[cold]
    pub fn build(kind: ErrorKind, code: u32, message: String, source: Option<Box<dyn Error + Send + Sync>>) -> Self {
        Self {
            code,
            kind,
            message,
            location: Location::caller(),
            source,
        }
    }
}

// --- EXTENSION TRAIT (GAT V2) ---

pub trait ErrorContextExt<T> {
    type Out<'a> where Self: 'a;

    fn with_lazy_context<F, S>(self, f: F) -> Self::Out<'static>
    where
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>;
}

impl<T, E> ErrorContextExt<T> for std::result::Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    type Out<'a> = Result<T> where Self: 'a;

    fn with_lazy_context<F, S>(self, f: F) -> Self::Out<'static>
    where
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>,
    {
        self.map_err(|e| {
            let (kind, code, msg) = f();
            ErrorReport::build(kind, code, msg.into(), Some(Box::new(e)))
        })
    }
}

impl<T> ErrorContextExt<T> for Option<T> {
    type Out<'a> = Result<T> where Self: 'a;

    fn with_lazy_context<F, S>(self, f: F) -> Self::Out<'static>
    where
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>,
    {
        self.ok_or_else(|| {
            let (kind, code, msg) = f();
            ErrorReport::build(kind, code, msg.into(), None)
        })
    }
}

// --- MACROS D'USAGE ---

#[macro_export]
macro_rules! error {
    ($def:expr, $fmt:expr $(, $arg:tt)*) => {
        move || {
            let d = $def;
            (d.kind(), d.code(), format!($fmt $(, $arg)*))
        }
    };
}

#[macro_export]
macro_rules! bail {
    ($def:expr, $fmt:expr $(, $arg:tt)*) => {
        return Err($crate::error::ErrorReport::build(
            $def.kind(),
            $def.code(),
            format!($fmt $(, $arg)*),
            None
        ))
    };
}
