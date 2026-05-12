use core::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct DiagnosticId(pub u32);

impl DiagnosticId {
    pub const fn new(id: u32) -> Self {
        Self(id)
    }
}

impl core::fmt::Display for DiagnosticId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "E{:04X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RawDiagnostic {
    pub id: DiagnosticId,
    pub location: &'static core::panic::Location<'static>,
}

impl RawDiagnostic {
    #[track_caller] // <--- 1. Indispensable ici
    pub fn new(id: u32) -> Self {
        Self {
            id: DiagnosticId::new(id),
            location: std::panic::Location::caller(),
        }
    }
}

pub const trait AsDiagnosticId {
    const ID: u32;
}

pub struct Diagnostic<T = ()> {
    pub inner: RawDiagnostic,
    _marker: PhantomData<fn() -> T>, // fn() -> T évite les soucis de variance et de drop
}

impl<T> Diagnostic<T> {
    #[track_caller]
    pub const fn from_type() -> Self
    where
        T: AsDiagnosticId,
        [(); const { (T::ID <= 0xFFFF) as usize }]:, // Erreur de compile si ID > 0xFFFF
    {
        Self {
            inner: RawDiagnostic {
                id: DiagnosticId::new(T::ID),
                location: std::panic::Location::caller(),
            },
            _marker: PhantomData,
        }
    }

    /// 2. APPROCHE DYNAMIQUE (Runtime)
    /// Pour les IDs générés à la volée ou provenant de l'extérieur.
    #[track_caller]
    pub fn manual(id: u32) -> Self {
        Self {
            inner: RawDiagnostic::new(id),
            _marker: PhantomData,
        }
    }
}


#[macro_export]
macro_rules! define_modules {
    (
        // On capture la visibilité (ex: pub), le nom de l'enum, et les variantes avec leurs valeurs
        $vis:vis enum $name:ident {
            $($variant:ident => $val:expr),* $(,)?
        }
    ) => {
        // On applique automatiquement les traits requis pour l'utilisation en const generic
        #[derive(Debug, Clone, Copy, PartialEq, Eq, core::marker::ConstParamTy)]
        #[repr(u32)] // Essentiel pour le décalage de bits (bit-shifting) dans ModuleDiagnostic
        $vis enum $name {
            $(
                $variant = $val
            ),*
        }

        // Optionnel : On peut générer des méthodes utilitaires pour l'enum
        impl $name {
            /// Retourne l'identifiant numérique du module
            pub const fn id(&self) -> u32 {
                *self as u32
            }
            
            /// Retourne le nom du module sous forme de chaîne de caractères
            pub const fn name(&self) -> &'static str {
                match self {
                    $(Self::$variant => stringify!($variant)),*
                }
            }
        }
    };
}


pub struct ModuleDiagnostic<const M: StoreModule, T> {
    pub inner: Diagnostic<T>,
    _marker: PhantomData<fn() -> T>,
}

impl<const M: StoreModule, T> ModuleDiagnostic<M, T> 
where 
    T: AsDiagnosticId,
    [(); const { (T::ID <= 0xFFFF) as usize }]:,
{
    #[track_caller]
    pub const fn new() -> Self {
        Self {
            inner: Diagnostic::<T>::from_type(),
            _marker: PhantomData,
        }
    }
}