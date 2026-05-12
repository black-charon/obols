use core::{marker::PhantomData, panic::Location};

pub const trait AsDiagnosticId {
    type Id: Sized + Copy;

    const ID: Self::Id;

    fn as_u32() -> u32;
}

#[repr(transparent)]
pub struct WrapperDiagnosticId<I>
where
    I: const AsDiagnosticId,
    [(); const { (I::as_u32() < 0xFFFF) as usize }]:,
{
    pub value: u32,
    _marker: PhantomData<I>,
}

#[derive(Debug, Clone, Copy)]
pub struct RawDiagnostic<'a, const ID: u32>
where
    [(); const { (ID < 0xFFFF) as usize }]:,
{
    pub code: u32,
    pub location: std::panic::Location<'a>,
}

impl<'a, const ID: u32> RawDiagnostic<'a, ID>
where
    [(); const { (ID < 0xFFFF) as usize }]:,
{
    #[track_caller]
    pub fn new() -> Self {
        Self {
            location: *Location::caller(),
            code: ID,
        }
    }
}

pub struct Diagnostic<'a, T>
where
    T: const AsDiagnosticId,
    [(); const { (T::as_u32() < 0xFFFF) as usize }]:,
{
    pub inner: RawDiagnostic<'a, { T::as_u32() }>, 
    _marker: PhantomData<T>, // Utilisation stricte de core::marker::PhantomData
}

impl<'a, T> Diagnostic<'a, T>
where
    T: const AsDiagnosticId,
    [(); const { (T::as_u32() < 0xFFFF) as usize }]:,
{
    #[track_caller]
    pub fn new() -> Self {
        Self {
            inner: RawDiagnostic::new(),
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    // Import de std uniquement pour les macros de test comme println!
    extern crate std; 
    use super::*;

    #[derive(Debug, Clone, Copy)]
    pub enum MyError {
        VariantA,
        VariantB,
    }

    impl const AsDiagnosticId for MyError {
        type Id = u32;

        const ID: Self::Id = 0x1234;

        fn as_u32() -> u32 {
            Self::ID
        }
    }

    #[test]
    fn test_diagnostic() {
        // L'appel est maintenant beaucoup plus élégant :
        // Plus besoin de dupliquer l'ID <0x1234, MyError> !
        let diag = Diagnostic::<MyError>::new();
        
        assert_eq!(diag.inner.code, 0x1234);
        std::println!("Diagnostic created at: {:?}", diag.inner.location);
    }
}