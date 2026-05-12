use crate::diagnostic::Diagnostic;

pub struct ErrorReport<'a, T> {
    pub diagnostics: Diagnostic<'a, T>,
}
