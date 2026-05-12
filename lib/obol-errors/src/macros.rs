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
                pub const fn emit() -> $crate::diagnostic::ModuleDiagnostic<{$module}, Self> {
                    // Ici aussi, on utilise $crate pour la cohérence
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
                    $(Self::$variant => $crate::report::ErrorReport::new($variant::emit().inner)),*
                }
            }
        }
    };
}

#[macro_export]
macro_rules! error {
   // V1 : Clé => Valeur
    ($result:expr, $key:expr => $val:expr) => {
        $crate::context::ErrorContextExt::with_data($result, $key, $val)
    };

    // V2 : Message formaté (style anyhow)
    ($result:expr, $($arg:tt)*) => {
        $crate::context::ErrorContextExt::with_context(
            $result, 
            format!($($arg)*)
        )
    };
}