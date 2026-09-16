// Realistic Rust test file 7
#[cfg(test)]
mod tests_7 {
    #[test]
    fn test_computation_7() {
        let x = 7 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (7 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_7() {
        let text = format!("item_7");
        assert_eq!(text.len(), 6);
        assert!(!text.is_empty());
    }
}
