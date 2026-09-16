// Realistic Rust test file 8
#[cfg(test)]
mod tests_8 {
    #[test]
    fn test_computation_8() {
        let x = 8 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (8 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_8() {
        let text = format!("item_8");
        assert_eq!(text.len(), 6);
        assert!(!text.is_empty());
    }
}
