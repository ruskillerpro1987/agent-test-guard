// Realistic Rust test file 4
#[cfg(test)]
mod tests_4 {
    #[test]
    fn test_computation_4() {
        let x = 4 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (4 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_4() {
        let text = format!("item_4");
        assert_eq!(text.len(), 6);
        assert!(!text.is_empty());
    }
}
