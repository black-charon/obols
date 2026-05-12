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


#[derive(PartialEq, Eq, core::marker::ConstParamTy)]
pub enum StoreModule {
    Inventory, // Gestion des stocks
    Checkout,  // Panier et paiement
    Identity,  // Utilisateurs et auth
    Shipping,  // Logistique
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