
///
#[derive(Debug)]
#[repr(transparent)]
pub struct Sku<const N: usize = 16>(pub [u8; N]);

impl<const N: usize> core::fmt::Display for Sku<N> {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "")
    }
}