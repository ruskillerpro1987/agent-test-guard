// Realistic Rust test file 13
#[cfg(test)]
mod tests_13 {
    #[test]
    fn test_computation_13() {
        let x = 13 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (13 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_13() {
        let text = format!("item_13");
        assert_eq!(text.len(), 7);
        assert!(!text.is_empty());
    }
}
