use crate::{context::ContextValue, diagnostic::RawDiagnostic, prelude::*};
use std::collections::HashMap;

pub struct ErrorReport {
    pub primary: RawDiagnostic,
    pub details: Vec<RawDiagnostic>,
    pub context: HashMap<String, ContextValue>,
}

impl ErrorReport {
    pub fn new(primary: RawDiagnostic) -> Self {
        Self {
            primary,
            details: Vec::new(),
            context: HashMap::new(),
        }
    }

    pub fn with_note<T>(mut self, diag: Diagnostic<T>) -> Self {
        self.details.push(diag.inner);
        self
    }

    /// Méthode utilitaire pour ajouter une note manuelle rapidement
    #[track_caller]
    pub fn add_note(mut self, id: u32) -> Self {
        self.details.push(RawDiagnostic::new(id));
        self
    }

    pub fn add_data<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<ContextValue>,
    {
        self.context.insert(key.into(), value.into());
        self
    }
}

// Permet la conversion de Diagnostic -> ErrorReport
impl<T> From<crate::diagnostic::Diagnostic<T>> for ErrorReport {
    fn from(diag: crate::diagnostic::Diagnostic<T>) -> Self {
        Self::new(diag.inner)
    }
}

// Permet la conversion de ModuleDiagnostic -> ErrorReport
impl<const M: crate::diagnostic::StoreModule, T: crate::diagnostic::AsDiagnosticId>
    From<crate::diagnostic::ModuleDiagnostic<M, T>> for ErrorReport
{
    fn from(md: crate::diagnostic::ModuleDiagnostic<M, T>) -> Self {
        Self::new(md.inner.inner)
    }
}

impl core::fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Affichage de l'erreur principale
        writeln!(f, "error: primary diagnostic E{:X}", self.primary.id.0)?;
        writeln!(
            f,
            "  --> {}:{}:{}",
            self.primary.location.file(),
            self.primary.location.line(),
            self.primary.location.column()
        )?;

        // Affichage des détails/notes
        for detail in &self.details {
            writeln!(f, "  note: additional context E{:X}", detail.id.0)?;
            writeln!(
                f,
                "    at {}:{}",
                detail.location.file(),
                detail.location.line()
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::diagnostic::DiagnosticId;

    use super::*;

    // Types d'erreurs pour les tests
    struct ErrMain;
    impl AsDiagnosticId for ErrMain {
        const ID: u32 = 0x1111;
    }

    struct ErrNote;
    impl AsDiagnosticId for ErrNote {
        const ID: u32 = 0x2222;
    }

    #[test]
    fn test_diagnostic_id_display() {
        let id = DiagnosticId::new(0xABC);
        assert_eq!(format!("{}", id), "E0ABC");
    }

    #[test]
    fn test_report_structure() {
        let primary = Diagnostic::<ErrMain>::from_type();
        let mut report = ErrorReport::new(primary.inner);

        report = report.with_note(Diagnostic::<ErrNote>::from_type());
        report = report.add_note(0x3333);

        assert_eq!(report.primary.id, DiagnosticId::new(0x1111));
        assert_eq!(report.details.len(), 2);
        assert_eq!(report.details[0].id, DiagnosticId::new(0x2222));
        assert_eq!(report.details[1].id, DiagnosticId::new(0x3333));
    }

    #[test]
    fn test_manual_diagnostic() {
        let diag = Diagnostic::<ErrMain>::manual(0x9999);
        assert_eq!(diag.inner.id, DiagnosticId::new(0x9999));
    }

    #[test]
    fn test_track_caller_localization() {
        let diag = Diagnostic::<ErrMain>::from_type();
        // Vérifie simplement que la localisation n'est pas celle de la définition (ex: ligne 50-60)
        // mais bien celle de cet appel (ligne > 90)
        assert!(diag.inner.location.line() > 80);
    }

    #[test]
    fn test_report_hierarchy() {
        let report = ErrorReport::new(Diagnostic::<ErrMain>::from_type().inner).add_note(0x2222);

        assert_eq!(report.primary.id, DiagnosticId::new(0x1111));
        assert_eq!(report.details.len(), 1);
    }

    #[test]
    fn test_display_output() {
        let primary = Diagnostic::<ErrMain>::from_type().inner;
        let report = ErrorReport::new(primary)
            .add_note(0x2222);
        let output = format!("{}", report);
        println!("{}", output);
        assert!(output.contains("error: primary diagnostic E1111"));
        assert!(output.contains("error: additional context E2222"));
    }
}
