use crate::{context::ContextValue, diagnostic::RawDiagnostic, prelude::*};
use std::collections::HashMap;
use std::fmt; // Changed to std::fmt for consistency

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

impl fmt::Display for ErrorReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Affichage de l'erreur principale
        writeln!(f, "error: primary diagnostic E{:04X}", self.primary.id.0)?; // Added :04X for consistent padding
        writeln!(
            f,
            "  --> {}:{}:{}",
            self.primary.location.file(),
            self.primary.location.line(),
            self.primary.location.column()
        )?;

        // Affichage des détails/notes
        for detail in &self.details {
            writeln!(f, "  note: additional context E{:04X}", detail.id.0)?;
            writeln!(
                f,
                "    at {}:{}",
                detail.location.file(),
                detail.location.line()
            )?;
        }

        // NOUVEAU: Affichage du contexte
        if !self.context.is_empty() {
            writeln!(f, "  context:")?;
            for (key, value) in &self.context {
                writeln!(f, "    | {} = {}", key, value)?;
            }
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

    /* ... [keep your other tests exactly the same] ... */

    #[test]
    fn test_display_output() {
        let primary = Diagnostic::<ErrMain>::from_type().inner;
        let report = ErrorReport::new(primary)
            .add_note(0x2222)
            .add_data("user_id", 42); // Testing the context output too!
            
        let output = format!("{}", report);
        println!("{}", output);
        
        assert!(output.contains("error: primary diagnostic E1111"));
        // FIX: Changed "error:" to "note:" to match the Display impl
        assert!(output.contains("note: additional context E2222")); 
        assert!(output.contains("user_id = 42"));
    }
}
