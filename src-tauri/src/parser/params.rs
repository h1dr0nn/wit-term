//! CSI parameter parsing utilities.

/// Parsed CSI parameters.
#[derive(Debug, Clone, Default)]
pub struct Params {
    values: Vec<u16>,
}

impl Params {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Get parameter at index, returning default value if missing or zero.
    pub fn get(&self, index: usize, default: u16) -> u16 {
        match self.values.get(index) {
            Some(&v) if v > 0 => v,
            _ => default,
        }
    }

    /// Get raw parameter at index.
    pub fn get_raw(&self, index: usize) -> Option<u16> {
        self.values.get(index).copied()
    }

    /// Number of parameters.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether there are no parameters.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Get all values as a slice.
    pub fn as_slice(&self) -> &[u16] {
        &self.values
    }

    /// Parse parameters from raw CSI parameter bytes.
    pub fn from_raw(raw: &[u16]) -> Self {
        Self {
            values: raw.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_defaults() {
        let params = Params::new();
        assert_eq!(params.get(0, 1), 1);
        assert_eq!(params.get(1, 0), 0);
        assert!(params.is_empty());
    }

    #[test]
    fn test_params_values() {
        let params = Params::from_raw(&[5, 10, 0]);
        assert_eq!(params.get(0, 1), 5);
        assert_eq!(params.get(1, 1), 10);
        assert_eq!(params.get(2, 1), 1); // 0 returns default
        assert_eq!(params.get(3, 99), 99); // missing returns default
        assert_eq!(params.len(), 3);
    }
}
