//! # Obol Error System
//!
//! Ce module fournit une infrastructure d'erreur de grade industriel pour l'écosystème Obol.
//! Il combine la flexibilité de la `std` avec la rigueur des systèmes bas-niveau via :
//!
//! - **Codes Hexadécimaux** : Identifiants uniques pour le monitoring et le debugging.
//! - **GAT v2 (Nightly 1.97)** : Gestion précise des durées de vie via `ErrorContextExt`.
//! - **Capture de Précision** : Utilisation de `track_caller` pour localiser l'erreur.
//! - **Idiomes Rust** : API familière inspirée de `anyhow` mais avec une typologie stricte.

use core::fmt;
use core::error::{Error, Request};
use core::panic::Location;

// --- REGISTRE DES ERREURS ---

/// Définit les capacités d'un type pouvant servir de diagnostic d'erreur.
pub trait Diagnostic {
    /// Retourne le code hexadécimal unique de l'erreur (ex: 0x1001).
    fn code(&self) -> u32;
    /// Retourne la catégorie métier de l'erreur.
    fn kind(&self) -> ErrorKind;
}

/// Macro déclarative pour enregistrer des erreurs de façon centralisée.
macro_rules! register_errors {
    (
        $name:ident {
            $($variant:ident => ($code:expr, $kind:ident)),* $(,)?
        }
    ) => {
        #[doc = "Enumération générée des définitions d'erreurs."]
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $( #[doc = "Erreur de type"] $variant ),*
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

// Enregistrement des domaines par défaut
register_errors! {
    ErrorDef {
        /// Erreur de validation de données (0x1xxx).
        InvalidSku     => (0x1001, Validation),
        /// Erreur liée aux montants ou devises.
        InvalidPrice   => (0x1002, Validation),
        /// Rupture de stock ou inventaire manquant (0x2xxx).
        StockEmpty     => (0x2001, Trade),
        /// Échec de la transaction monétaire.
        PaymentFailed  => (0x2002, Trade),
        /// Corruption du Write-Ahead Log (0x3xxx).
        WalCorrupted   => (0x3001, Database),
        /// Erreur imprévue du système (0xFxxx).
        InternalError  => (0xF001, Internal),
    }
}

// --- TYPES DE BASE ---

/// Alias Result universel pour l'écosystème Obol.
pub type Result<T, E = ErrorReport> = core::result::Result<T, E>;

/// Catégories majeures d'erreurs pour le routage.
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

/// Structure principale transportant l'information d'erreur.
#[derive(Debug)]
pub struct ErrorReport {
    /// Code diagnostic unique (ex: 0x10A2).
    pub code: u32,
    /// Catégorie de l'erreur.
    pub kind: ErrorKind,
    /// Message contextuel décrivant l'échec.
    pub message: String,
    /// Localisation exacte de la levée de l'erreur dans le code source.
    pub location: &'static Location<'static>,
    /// Erreur parente ayant causé cet échec.
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
    /// Construit un nouveau rapport d'erreur.
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

/// Trait d'extension pour ajouter du contexte aux `Result` et `Option`.
pub trait ErrorContextExt<T> {
    /// Type de sortie utilisant les Generic Associated Types.
    type Out<'a> where Self: 'a;

    /// Ajoute un contexte d'erreur de façon paresseuse. La closure n'est exécutée
    /// qu'en cas d'erreur.
    ///
    /// # Exemple
    /// ```
    /// let sku = parse(input).with_context(error!(ErrorDef::InvalidSku, "Format incorrect"));
    /// ```
    fn with_context<F, S>(self, f: F) -> Self::Out<'static>
    where
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>;
}

impl<T, E> ErrorContextExt<T> for std::result::Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    type Out<'a> = Result<T> where Self: 'a;

    fn with_context<F, S>(self, f: F) -> Self::Out<'static>
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

    fn with_context<F, S>(self, f: F) -> Self::Out<'static>
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

/// Prépare une closure d'erreur pour `with_context`.
#[macro_export]
macro_rules! error {
    ($def:expr, $fmt:expr $(, $arg:tt)*) => {
        move || {
            let d = $def;
            (d.kind(), d.code(), format!($fmt $(, $arg)*))
        }
    };
}

/// Provoque une sortie immédiate de la fonction avec une erreur Obol.
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
