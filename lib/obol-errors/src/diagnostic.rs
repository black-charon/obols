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

pub struct Diagnostic<'a, const ID: u32, T>
where
    T: const AsDiagnosticId,
    [(); const { (ID < 0xFFFF) as usize }]:,
{
    /// Les données brutes liées à l'ID constant de T
    pub inner: RawDiagnostic<'a, ID>,
    /// Un marqueur pour conserver le lien avec le type d'erreur d'origine
    _marker: std::marker::PhantomData<T>,
}

impl<'a, const ID: u32, T> Diagnostic<'a, ID, T>
where
    T: const AsDiagnosticId,
    [(); const { (ID < 0xFFFF) as usize }]:,
{
    #[track_caller]
    pub fn new() -> Self {
        Self {
            inner: RawDiagnostic::new(),
            _marker: std::marker::PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic() {
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
        let diag = Diagnostic::<0x1234, MyError>::new();
        assert_eq!(diag.inner.code, 0x1234);
        println!("Diagnostic created at: {:?}", diag.inner.location);
    }
}
