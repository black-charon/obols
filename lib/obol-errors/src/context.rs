use crate::report::ErrorReport;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum ContextValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    None,
}

// 1. Keep your From implementations (they are perfect)
impl From<String> for ContextValue { fn from(s: String) -> Self { Self::Str(s) } }
impl From<&str> for ContextValue { fn from(s: &str) -> Self { Self::Str(s.to_string()) } }
impl From<i64> for ContextValue { fn from(i: i64) -> Self { Self::Int(i) } }
impl From<f64> for ContextValue { fn from(f: f64) -> Self { Self::Float(f) } }
impl From<bool> for ContextValue { fn from(b: bool) -> Self { Self::Bool(b) } }

impl<T: Into<ContextValue>> From<Option<T>> for ContextValue {
    fn from(opt: Option<T>) -> Self {
        match opt {
            Some(v) => v.into(),
            None => Self::None,
        }
    }
}

impl std::fmt::Display for ContextValue {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => write!(fmt, "{}", s),
            Self::Int(i) => write!(fmt, "{}", i),
            Self::Float(f) => write!(fmt, "{}", f),
            Self::Bool(b) => write!(fmt, "{}", b),
            Self::None => write!(fmt, "null"),
        }
    }
}

pub struct ErrorContext {
    pub metadata: HashMap<String, ContextValue>,
}

// 2. We got rid of ErrorContextTrait entirely!

pub trait ErrorContextExt<T> {
    /// Eagerly adds context data. Best for cheap values (e.g., integers, &str).
    fn with_data<V>(self, key: &'static str, value: V) -> Result<T, ErrorReport>
    where
        V: Into<ContextValue>;

    /// Lazily adds context data via a closure. Best for expensive operations (e.g., format!).
    fn with_data_lazy<F, V>(self, key: &'static str, f: F) -> Result<T, ErrorReport>
    where
        F: FnOnce() -> V,
        V: Into<ContextValue>;

    /// Eagerly adds a simple message.
    fn with_context<V>(self, value: V) -> Result<T, ErrorReport>
    where
        V: Into<ContextValue>;
        
    /// Lazily adds a simple message.
    fn with_context_lazy<F, V>(self, f: F) -> Result<T, ErrorReport>
    where
        F: FnOnce() -> V,
        V: Into<ContextValue>;
}

impl<T, E> ErrorContextExt<T> for Result<T, E>
where
    E: Into<ErrorReport>,
{
    fn with_data<V>(self, key: &'static str, value: V) -> Result<T, ErrorReport>
    where
        V: Into<ContextValue>,
    {
        self.map_err(|e| {
            let mut report: ErrorReport = e.into();
            // Note: Make sure to access `.metadata` if `context` is your `ErrorContext` struct
            report.context.metadata.insert(key.to_string(), value.into());
            report
        })
    }

    fn with_data_lazy<F, V>(self, key: &'static str, f: F) -> Result<T, ErrorReport>
    where
        F: FnOnce() -> V,
        V: Into<ContextValue>,
    {
        self.map_err(|e| {
            let mut report: ErrorReport = e.into();
            report.context.metadata.insert(key.to_string(), f().into());
            report
        })
    }

    fn with_context<V>(self, value: V) -> Result<T, ErrorReport>
    where
        V: Into<ContextValue>,
    {
        self.with_data("message", value)
    }

    fn with_context_lazy<F, V>(self, f: F) -> Result<T, ErrorReport>
    where
        F: FnOnce() -> V,
        V: Into<ContextValue>,
    {
        self.with_data_lazy("message", f)
    }
}
