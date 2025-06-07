use rand::{Rng, SeedableRng, rngs::StdRng};
use serde_json::{Value, json};

/// Generate n random contexts to use in the benchmark
pub fn generate_random_contexts(n: usize) -> Vec<Value> {
    let mut rng = StdRng::seed_from_u64(42); // Fixed seed for reproducibility
    let mut contexts = Vec::with_capacity(n);

    for _ in 0..n {
        let name = random_string(&mut rng, 5, 10);
        let age = rng.gen_range(18..80);
        let is_active = rng.gen_bool(0.7);

        let items_count = rng.gen_range(3..10);
        let mut items = Vec::with_capacity(items_count);
        for _ in 0..items_count {
            let item_name = random_string(&mut rng, 3, 8);
            let item_value = rng.gen_range(10..1000);
            items.push(json!({
                "name": item_name,
                "value": item_value,
                "special": rng.gen_bool(0.3)
            }));
        }

        contexts.push(json!({
            "user": {
                "name": name,
                "age": age,
                "active": is_active
            },
            "items": items,
            "show_details": rng.gen_bool(0.8),
            "has_access": rng.gen_bool(0.6),
        }));
    }

    contexts
}

/// Generate a random string with length between min and max
fn random_string(rng: &mut StdRng, min_len: usize, max_len: usize) -> String {
    let charset = "abcdefghijklmnopqrstuvwxyz";
    let len = rng.gen_range(min_len..=max_len);

    (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect()
}

// Print binary size information - can be used from individual benchmarks
pub fn print_binary_size() {
    let binary_path = std::env::current_exe().unwrap();
    let metadata = std::fs::metadata(binary_path.clone()).unwrap();
    let size_bytes = metadata.len();
    let size_kb = size_bytes as f64 / 1024.0;
    let size_mb = size_kb / 1024.0;

    println!(
        "Binary size: {:.2} MB ({:.2} KB, {} bytes)",
        size_mb, size_kb, size_bytes
    );
    println!("Binary path: {}", binary_path.display());
}
