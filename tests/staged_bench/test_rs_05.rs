// Realistic Rust test file 5
#[cfg(test)]
mod tests_5 {
    #[test]
    fn test_computation_5() {
        let x = 5 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (5 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_5() {
        let text = format!("item_5");
        assert_eq!(text.len(), 6);
        assert!(!text.is_empty());
    }
}
