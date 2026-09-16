// Realistic Rust test file 2
#[cfg(test)]
mod tests_2 {
    #[test]
    fn test_computation_2() {
        let x = 2 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (2 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_2() {
        let text = format!("item_2");
        assert_eq!(text.len(), 6);
        assert!(!text.is_empty());
    }
}
