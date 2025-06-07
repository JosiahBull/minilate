use minilate::{MinilateEngine, MinilateInterface};
use rand::Rng;

pub fn get_engine() -> impl MinilateInterface {
    MinilateEngine::new()
}

pub fn generate_random_whitespace() -> String {
    let mut rng = rand::thread_rng();
    let length = rng.gen_range(0..10);
    (0..length).map(|_| ' ').collect()
}

pub fn generate_random_whitespace_at_least_one() -> String {
    let mut rng = rand::thread_rng();
    let length = rng.gen_range(1..10);
    (0..length).map(|_| ' ').collect()
}
