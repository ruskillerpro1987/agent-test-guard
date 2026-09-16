// Realistic Rust test file 17
#[cfg(test)]
mod tests_17 {
    #[test]
    fn test_computation_17() {
        let x = 17 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (17 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_17() {
        let text = format!("item_17");
        assert_eq!(text.len(), 7);
        assert!(!text.is_empty());
    }
}
