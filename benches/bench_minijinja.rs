use criterion::{Criterion, black_box, criterion_group, criterion_main};
use minijinja::Environment;

mod utils;

fn minijinja_benchmark(c: &mut Criterion) {
    // Create the MiniJinja environment
    let mut env = Environment::new();

    // Load the template from file
    let template_content = include_str!("template_minijinja.jinja");

    // Add template to environment
    env.add_template("profile", template_content).unwrap();

    // Generate 100 random contexts
    let contexts = utils::generate_random_contexts(100);

    // Print binary size information
    utils::print_binary_size();

    // Setup benchmark group
    let mut group = c.benchmark_group("Template Rendering");
    group.sample_size(50);

    // Benchmark template rendering
    group.bench_function("minijinja_render", |b| {
        b.iter(|| {
            let template = env.get_template("profile").unwrap();
            for context in &contexts {
                black_box(template.render(context).unwrap());
            }
        });
    });

    group.finish();
}

criterion_group!(benches, minijinja_benchmark);
criterion_main!(benches);
