use proptest::prelude::*;
proptest! {
    #[test]
    fn hash_always_64_chars(input in ".*") {
        let hash = generate_hash(&input);
        prop_assert_eq!(hash.len(), 64);
    }
}
