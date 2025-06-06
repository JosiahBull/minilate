use minilate::{MinilateEngine, MinilateInterface};

pub fn get_engine() -> impl MinilateInterface {
    MinilateEngine::new()
}
