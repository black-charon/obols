//! # Obol Error System
//!
//! Ce module fournit une infrastructure d'erreur de grade industriel pour l'écosystème Obol.
//! Il combine la flexibilité de la `std` avec la rigueur des systèmes bas-niveau via :
//!
//! - **Codes Hexadécimaux** : Identifiants uniques pour le monitoring et le debugging.
//! - **GAT v2 (Nightly 1.97)** : Gestion précise des durées de vie via `ErrorContextExt`.
//! - **Capture de Précision** : Utilisation de `track_caller` pour localiser l'erreur.
//! - **Idiomes Rust** : API familière inspirée de `anyhow` mais avec une typologie stricte.

use core::error::{Error, Request};
use core::fmt;
use core::panic::Location;

use serde::{Deserialize, Serialize};

/// Définit les capacités d'un type pouvant servir de diagnostic d'erreur.
pub trait Diagnostic {
    /// Retourne le code hexadécimal unique de l'erreur (ex: 0x1001).
    fn code(&self) -> u32;
    /// Retourne la catégorie métier de l'erreur.
    fn kind(&self) -> ErrorKind;
}

#[macro_export]
macro_rules! register_errors {
    (
        $(#[$enum_meta:meta])*
        $name:ident {
            $(
                $(#[$variant_meta:meta])* // Capture les doc comments ///
                $variant:ident => ($code:expr, $kind:ident)
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),*
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
        Database   => (0x3001, Database),
        /// Erreur imprévue du système (0xFxxx).
        InternalError  => (0xF001, Internal),

    }
}

// --- TYPES DE BASE ---

/// Alias Result universel pour l'écosystème Obol.
pub type Result<T, E = ErrorReport> = core::result::Result<T, E>;

/// Catégories majeures d'erreurs pour le routage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

/// Structure principale transportant l'information d'erreur.
#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorReport {
    /// Code diagnostic unique (ex: 0x10A2).
    pub code: u32,
    /// Catégorie de l'erreur.
    pub kind: ErrorKind,
    /// Message contextuel décrivant l'échec.
    pub message: String,
    /// Pour la sérialisation, on stocke le chemin en String
    pub file: String,
    pub line: u32,
    pub col: u32,
    /// L'erreur parente (ignorée par Serde car non-désérialisable facilement)
    #[serde(skip)]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    /// La référence de localisation originale (ignorée par Serde)
    #[serde(skip)]
    pub location: Option<&'static std::panic::Location<'static>>,
}

impl ErrorReport {
    /// Construit un nouveau rapport d'erreur.
    #[track_caller]
    #[cold]
    pub fn build(
        kind: ErrorKind,
        code: u32,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    ) -> Self {
        let caller = Location::caller();
        Self {
            code,
            kind,
            message,
            file: caller.file().to_string(),
            line: caller.line(),
            col: caller.column(),
            location: Some(caller),
            source,
        }
    }

    pub fn to_json_object(&self) -> serde_json::Value {
        serde_json::to_value(self)
            .unwrap_or_else(|_| serde_json::json!({ "error": "failed to serialize error report" }))
    }

    /// Transforme le rapport en un objet JSON sécurisé pour les clients externes.
    /// Filtre les informations sensibles comme la localisation du code.
    pub fn to_public_json_object(&self) -> serde_json::Value {
        serde_json::json!({
            "code": format!("{:#06X}", self.code),
            "kind": self.kind,
            "message": self.message,
        })
    }
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
        write!(
            f,
            "[ERR:{:#06X}] {} -> {}",
            self.code, self.kind, self.message
        )?;

        write!(f, " (at {}:{})", self.file, self.line)?;

        if let Some(ref src) = self.source {
            write!(f, " | Source: {}", src)?;
        }
        Ok(())
    }
}

// --- EXTENSION TRAIT (GAT V2) ---

/// Trait d'extension pour ajouter du contexte aux `Result` et `Option`.
pub trait ErrorContextExt<T> {
    type Out<'a>
    where
        Self: 'a;

    /// On définit 'a explicitement pour lier self et la sortie.
    #[track_caller]
    fn with_context<'a, F, S>(self, f: F) -> Self::Out<'a>
    where
        Self: 'a, // Crucial : indique que Self est valide pour 'a
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>;
}

impl<T, E> ErrorContextExt<T> for std::result::Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    type Out<'a>
        = Result<T>
    where
        Self: 'a;

    #[track_caller]
    fn with_context<'a, F, S>(self, f: F) -> Self::Out<'a>
    where
        Self: 'a,
        F: FnOnce() -> (ErrorKind, u32, S),
        S: Into<String>,
    {
        self.map_err(|e| {
            let (kind, code, msg) = f();
            // L'appel à build() capture l'appelant grâce au #[track_caller] du trait
            ErrorReport::build(kind, code, msg.into(), Some(Box::new(e)))
        })
    }
}

impl<T> ErrorContextExt<T> for Option<T> {
    type Out<'a>
        = Result<T>
    where
        Self: 'a;

    #[track_caller]
    fn with_context<'a, F, S>(self, f: F) -> Self::Out<'a>
    where
        Self: 'a,
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

#[macro_export]
macro_rules! bail {
    ($def:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {
        use crate::error::Diagnostic;
        return Err($crate::error::ErrorReport::build(
            $def.kind(),
            $def.code(),
            format!($fmt $(, $arg)*),
            None
        ))
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_registration() {
        // Vérifie que la macro register_errors! a bien généré les codes et catégories
        assert_eq!(ErrorDef::InvalidSku.code(), 0x1001);
        assert_eq!(ErrorDef::InvalidSku.kind(), ErrorKind::Validation);

        assert_eq!(ErrorDef::Database.code(), 0x3001);
        assert_eq!(ErrorDef::Database.kind(), ErrorKind::Database);
    }

    #[test]
    fn test_bail_macro() {
        fn produce_error() -> Result<()> {
            bail!(
                ErrorDef::InternalError,
                "une erreur critique s'est produite: {}",
                "CPU_OVERHEAT"
            );
        }

        let res = produce_error();
        assert!(res.is_err());

        let report = res.unwrap_err();
        assert_eq!(report.code, 0xF001);
        assert_eq!(report.kind, ErrorKind::Internal);
        assert!(
            report
                .message
                .contains("une erreur critique s'est produite: CPU_OVERHEAT")
        );

        // OPTION A: Vérifier via le champ String (Recommandé pour Obol)
        // C'est ce champ qui voyagera entre tes microservices.
        assert!(report.file.contains("error.rs"));
        assert!(report.line > 0);

        // OPTION B: Vérifier via l'Option location (si tu veux être ultra précis en local)
        // On doit unwrap() car c'est une Option désormais.
        assert!(report.location.unwrap().file().contains("error.rs"));
    }

    #[test]
    fn test_with_context_on_result() {
        // Simule une erreur standard (io::Error)
        let standard_io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");

        let res: Result<()> = Err(standard_io_err).with_context(error!(
            ErrorDef::Database,
            "échec du chargement de la table {}", "users"
        ));

        assert!(res.is_err());
        let report = res.unwrap_err();

        assert_eq!(report.code, 0x3001); // WalCorrupted ou Database
        assert!(report.source.is_some()); // Vérifie que l'erreur source est préservée
        assert!(
            report
                .message
                .contains("échec du chargement de la table users")
        );
    }

    #[test]
    fn test_with_context_on_option() {
        let none_val: Option<i32> = None;

        let res =
            none_val.with_context(error!(ErrorDef::StockEmpty, "article {} introuvable", 404));

        assert!(res.is_err());
        let report = res.unwrap_err();
        assert_eq!(report.code, 0x2001);
        assert_eq!(report.kind, ErrorKind::Trade);
        assert!(report.source.is_none()); // Pas de source pour une conversion d'Option
    }

    #[test]
    fn test_provide_api() {
        use core::error::request_value;

        let report = ErrorReport::build(ErrorKind::Validation, 0x1234, "test provide".into(), None);

        // Test de l'API de diagnostic via provide
        let code = request_value::<u32>(&report);
        let kind = request_value::<ErrorKind>(&report);

        assert_eq!(code, Some(0x1234));
        assert_eq!(kind, Some(ErrorKind::Validation));
    }

    #[test]
    fn test_to_json_object() {
        let report = ErrorReport::build(ErrorKind::Trade, 0x2001, "Rupture de stock".into(), None);

        let obj = report.to_json_object();

        // On vérifie que les champs critiques sont là
        assert_eq!(obj["code"], 0x2001);
        assert_eq!(obj["kind"], "Trade");
        assert!(obj["file"].is_string());

        // Test de la version publique (sécurité)
        let public_obj = report.to_public_json_object();
        assert_eq!(public_obj["code"], "0x2001");
        assert!(public_obj.get("file").is_none()); // On vérifie l'absence de données sensibles
        println!("{}", public_obj);
    }
}
