#![allow(
    clippy::string_slice,
    clippy::tests_outside_test_module,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "benchmark"
)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use handlebars::Handlebars;

mod utils;

fn handlebars_benchmark(c: &mut Criterion) {
    // Create the Handlebars registry
    let mut handlebars = Handlebars::new();

    // Load the template from file
    let template_content = include_str!("template_handlebars.hbs");

    // Register template
    handlebars
        .register_template_string("profile", template_content)
        .unwrap();

    // Generate 100 random contexts
    let contexts = utils::generate_random_contexts(100);

    // Print binary size information
    utils::print_binary_size();

    // Setup benchmark group
    let mut group = c.benchmark_group("Template Rendering");
    group.sample_size(50);

    // Benchmark template rendering
    group.bench_function("handlebars_render", |b| {
        b.iter(|| {
            for context in &contexts {
                black_box(handlebars.render("profile", context).unwrap());
            }
        });
    });

    group.finish();
}

criterion_group!(benches, handlebars_benchmark);
criterion_main!(benches);
