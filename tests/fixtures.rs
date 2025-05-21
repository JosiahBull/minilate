use minilate::{ExampleEngine, MinilateEngine};

pub fn get_engine() -> impl MinilateEngine {
    ExampleEngine::new()
}
