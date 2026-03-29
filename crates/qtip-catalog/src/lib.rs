#![forbid(unsafe_code)]

/// Returns the crate name used by downstream integration tests and wiring checks.
pub fn crate_id() -> &'static str {
    "qtip-catalog"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_id_is_stable() {
        assert_eq!(crate_id(), "qtip-catalog");
    }
}
