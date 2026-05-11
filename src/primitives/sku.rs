use crate::{ErrorDef, ErrorReport, bail, error::Result};

#[repr(transparent)]
#[derive(Debug)]
pub struct Sku<const N: usize = 16>([u8; N]);

impl<const N: usize> Sku<N> {
    pub fn new(input: &str) -> Result<Self> {
        if input.len() > N {
            bail!(
                ErrorDef::InvalidSku,
                "SKU trop long ({} octets, max {})",
                input.len(),
                N
            );
        }

        let mut data = [0u8; N];
        data[..input.len()].copy_from_slice(input.as_bytes());
        Ok(Self(data))
    }

    pub fn as_str(&self) -> &str {
        let len = self.0.iter().position(|&b| b == 0).unwrap_or(N);
        unsafe { std::str::from_utf8_unchecked(&self.0[..len]) }
    }

    pub fn to_normalized(&self) -> String {
        self.as_str().to_uppercase()
    }
}

impl<const N: usize> core::fmt::Display for Sku<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Permet d'utiliser Sku::<16>::default() pour obtenir un SKU vide.
impl<const N: usize> Default for Sku<N> {
    fn default() -> Self {
        Self([0u8; N])
    }
}

// Permet la conversion facile : "mon-sku".try_into()?
impl<const N: usize> TryFrom<&str> for Sku<N> {
    type Error = ErrorReport;
    fn try_from(value: &str) -> Result<Self> {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_sku_industrial() {
        // Test de création valide
        let sku = Sku::<8>::new("APPLE").unwrap();
        assert_eq!(sku.as_str(), "APPLE");
        assert_eq!(format!("{sku}"), "APPLE");

        // Test de limite stricte
        let err = Sku::<4>::new("BANANA");
        assert!(err.is_err());
        assert_eq!(err.unwrap_err().code, 0x1001); // Notre code InvalidSku

        // Test du padding (repr transparent)
        let sku_pad = Sku::<8>::new("HI").unwrap();
        assert_eq!(sku_pad.0[0], b'H');
        assert_eq!(sku_pad.0[2], 0); // Padding nul
        println!("{}", sku)
    }
}

