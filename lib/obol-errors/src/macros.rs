#[macro_export]
macro_rules! new_error {
    (
        module = $module:expr,
        $name:ident {
            $($variant:ident => $code:expr),* $(,)?
        }
    ) => {
        // Définition de l'enum de confort
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $($variant),*
        }

        $(
            #[allow(non_camel_case_types)]
            pub struct $variant;

            // Utilisation du chemin absolu via $crate
            impl const $crate::diagnostic::AsDiagnosticId for $variant {
                const ID: u32 = $code;
            }

  impl $variant {
    #[track_caller]
    // MAGIE : On passe l'ID du module en le castant as u32 directement dans le const generic !
    pub const fn emit() -> $crate::diagnostic::ModuleDiagnostic<{ $module as u32 }, Self> {
        $crate::diagnostic::ModuleDiagnostic::new()
    }
}
        )*

        impl $name {
            pub const fn code(&self) -> u32 {
                match self {
                    $(Self::$variant => $code),*
                }
            }

            // Permet de transformer l'enum en rapport d'erreur complet
            #[track_caller]
            pub fn report(&self) -> $crate::report::ErrorReport {
                match self {
                    // FIX: Utilisation de .into() pour profiter de l'implémentation From
                    $(Self::$variant => $variant::emit().into()),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! context { // FIX: Renommé pour éviter le conflit avec log::error!
   // V1 : Clé => Valeur
    ($result:expr, $key:expr => $val:expr) => {
        $crate::context::ErrorContextExt::with_data($result, $key, $val)
    };

    // V2 : Message formaté (style anyhow)
    ($result:expr, $($arg:tt)*) => {
        // FIX: Utilisation de la version lazy pour protéger les performances du chemin Ok
        $crate::context::ErrorContextExt::with_context_lazy(
            $result, 
            || format!($($arg)*)
        )
    };
}
