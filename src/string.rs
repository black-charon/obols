#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct String<const N: usize> {
    data: [u8; N],
    len: usize,
}

impl<const N: usize> String<N> {
    /// Initialize constante
    pub const fn new() -> Self {
        Self {
            data: [0; N],
            len: 0,
        }
    }

    pub const fn from_static_str(s: &str) -> Self {
        assert!(s.len() <= N, "String Overflow");
        let s = s.as_bytes();
        let mut data = [0; N];
        let mut i = 0;
        while i < s.len() {
            data[i] = s[i];
            i += 1
        }
        Self { data, len: s.len() }
    }

    pub fn try_from_str(s: &str) -> Result<Self, &'static str> {
        if s.len() > N {
            return Err("Capacity exceeded");
        }
        let mut data = [0; N];
        data[..s.len()].copy_from_slice(s.as_bytes());
        Ok(Self { data, len: s.len() })
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        unsafe { core::str::from_utf8_unchecked(&self.data[..self.len]) }
    }
}

impl<const N: usize> core::ops::Deref for String<N> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> Default for String<N> {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
}