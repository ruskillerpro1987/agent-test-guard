// Realistic Rust test file 3
#[cfg(test)]
mod tests_3 {
    #[test]
    fn test_computation_3() {
        let x = 3 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (3 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_3() {
        let text = format!("item_3");
        assert_eq!(text.len(), 6);
        assert!(!text.is_empty());
    }
}
