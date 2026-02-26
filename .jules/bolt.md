## 2025-05-15 - [Regex Memoization]
**Learning:** Compiling regexes in frequently called functions (like PR diff scanning) is a significant bottleneck in Rust. Using `std::sync::OnceLock` to compile them only once resulted in a ~11x speedup.
**Action:** Always check for `Regex::new` in loops or core processing paths and move them to static `OnceLock` instances.

## 2025-05-15 - [Reqwest Client Reuse]
**Learning:** Creating a new `reqwest::Client` on every request is inefficient as it misses out on connection pooling and involves unnecessary allocations.
**Action:** Share a single `reqwest::Client` instance across the application, either via `AppState` or a static `OnceLock`.
