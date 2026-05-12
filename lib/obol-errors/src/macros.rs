
///
#[macro_export]
macro_rules! new_module_error {
    (
        $(#[$enum_meta:meta])*
        $name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => ($code:expr, $kind:ident)
            ),* $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum ErrorKind {
            $($kind),*
        }

        $(#[$enum_meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $name {
            $(
                $(#[$variant_meta])*
                $variant
            ),*
        }

        impl $name {
            pub const fn kind(&self) -> ErrorKind {
                match self {
                    $(Self::$variant => ErrorKind::$kind),*
                }
            }

            pub const fn code(&self) -> u32 {
                match self {
                    $(Self::$variant => {
                        // Validation stricte : ID doit être < 0xFFFF
                        let _ = $crate::diagnostic::CodeDiagnostic::<$code>::new();
                        $code
                    }),*
                }
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "[{:04X}] {:?} ({:?})", self.code(), self, self.kind())
            }
        }
    };
}

#[macro_export]
macro_rules! error {
    ($def:expr, $fmt:expr, $(, $arg:tt)*) => {
        move || {}
    };
}

#[macro_export]
macro_rules! bail {
    ($def:expr, $fmt:expr $(, $arg:expr)* $(,)?) => {};
}
