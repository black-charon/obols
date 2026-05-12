use crate::report::ErrorReport;

#[derive(Debug, Clone)]
pub enum ContextValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    None,
}

impl From<String> for ContextValue {
    fn from(s: String) -> Self {
        Self::Str(s)
    }
}
impl From<&str> for ContextValue {
    fn from(s: &str) -> Self {
        Self::Str(s.to_string())
    }
}
impl From<i64> for ContextValue {
    fn from(i: i64) -> Self {
        Self::Int(i)
    }
}
impl From<f64> for ContextValue {
    fn from(f: f64) -> Self {
        Self::Float(f)
    }
}
impl From<bool> for ContextValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl std::fmt::Display for ContextValue {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => write!(fmt, "{}", s),
            Self::Int(i) => write!(fmt, "{}", i),
            Self::Float(f) => write!(fmt, "{}", f),
            Self::Bool(b) => write!(fmt, "{}", b),
            Self::None => write!(fmt, "null"), // Affichage propre
        }
    }
}

pub struct ErrorContext {
    pub metadata: std::collections::HashMap<String, ContextValue>,
}

pub trait ErrorContextTrait: std::fmt::Debug + Send + Sync {
    fn to_context_value(self) -> ContextValue;
}

impl ErrorContextTrait for String {
    fn to_context_value(self) -> ContextValue {
        ContextValue::Str(self)
    }
}
impl ErrorContextTrait for &str {
    fn to_context_value(self) -> ContextValue {
        ContextValue::Str(self.to_string())
    }
}
impl ErrorContextTrait for i64 {
    fn to_context_value(self) -> ContextValue {
        ContextValue::Int(self)
    }
}
impl ErrorContextTrait for f64 {
    fn to_context_value(self) -> ContextValue {
        ContextValue::Float(self)
    }
}
impl ErrorContextTrait  for bool {
    fn to_context_value(self) -> ContextValue {
        ContextValue::Bool(self)
    }
}

impl<T> ErrorContextTrait for Option<T>
where
    T: ErrorContextTrait,
{
    fn to_context_value(self) -> ContextValue {
        match self {
            Some(value) => value.to_context_value(),
            None => ContextValue::None,
        }
    }
}

pub trait ErrorContextExt<T> {
    fn with_data<V>(self, key: &'static str, value: V) -> Result<T, ErrorReport>
    where
        V: ErrorContextTrait;

    /// Ajoute un message simple sous la clé "message" (Style Anyhow)
    fn with_context<V>(self, value: V) -> Result<T, ErrorReport>
    where
        V: ErrorContextTrait;
}

impl<T, E> ErrorContextExt<T> for Result<T, E>
where
    E: Into<ErrorReport>,
{
    fn with_data<V>(self, key: &'static str, value: V) -> Result<T, ErrorReport>
    where
        V: ErrorContextTrait,
    {
        self.map_err(|e| {
            let mut report: ErrorReport = e.into();
            report
                .context
                .insert(key.to_string(), value.to_context_value());
            report
        })
    }

    fn with_context<V>(self, value: V) -> Result<T, ErrorReport>
    where
        V: ErrorContextTrait,
    {
        // On redirige vers with_data avec une clé par défaut
        self.with_data("message", value)
    }
}
