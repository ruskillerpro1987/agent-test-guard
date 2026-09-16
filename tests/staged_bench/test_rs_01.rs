// Realistic Rust test file 1
#[cfg(test)]
mod tests_1 {
    #[test]
    fn test_computation_1() {
        let x = 1 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (1 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_1() {
        let text = format!("item_1");
        assert_eq!(text.len(), 6);
        assert!(!text.is_empty());
    }
}
