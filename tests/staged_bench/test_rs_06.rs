// Realistic Rust test file 6
#[cfg(test)]
mod tests_6 {
    #[test]
    fn test_computation_6() {
        let x = 6 * 2 + 10;
        let y = x * 3;
        assert_eq!(y, (6 * 2 + 10) * 3);
        assert!(y > 0);
    }

    #[test]
    fn test_validation_6() {
        let text = format!("item_6");
        assert_eq!(text.len(), 6);
        assert!(!text.is_empty());
    }
}
