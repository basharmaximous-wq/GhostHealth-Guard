use proptest::prelude::*;
use ghosthealth_guard::hash::generate_hash;

proptest! {
    #[test]
    fn hash_always_64_chars(input in ".*") {
        let hash = generate_hash(&input);
        prop_assert_eq!(hash.len(), 64);
    }
}