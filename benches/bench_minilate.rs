#![allow(
    clippy::string_slice,
    clippy::tests_outside_test_module,
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "benchmark"
)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use minilate::{Context, MinilateEngine, MinilateInterface, VariableTy};
use serde_json::Value;

mod utils;

fn minilate_benchmark(c: &mut Criterion) {
    // Create the Minilate engine
    let mut engine = MinilateEngine::new();

    // Load the template from file
    let template_content = include_str!("template_minilate.tmpl");

    // Add template to engine
    engine.add_template("profile", template_content).unwrap();

    // Generate 100 random contexts
    let json_contexts = utils::generate_random_contexts(100);

    // Convert JSON contexts to Minilate contexts
    let contexts: Vec<Context> = json_contexts
        .iter()
        .map(|json_ctx| create_minilate_context(json_ctx))
        .collect();

    // Print binary size information
    utils::print_binary_size();

    // Setup benchmark group
    let mut group = c.benchmark_group("Template Rendering");
    group.sample_size(50);

    // Benchmark template rendering
    group.bench_function("minilate_render", |b| {
        b.iter(|| {
            for context in &contexts {
                black_box(engine.render("profile", Some(context)).unwrap());
            }
        });
    });

    group.finish();
}

// Convert JSON data to Minilate context
fn create_minilate_context(json: &Value) -> Context<'static> {
    let mut context = Context::new();

    // User information
    let user_name = json["user"]["name"].as_str().unwrap().to_owned();
    let user_age = json["user"]["age"].as_i64().unwrap().to_string();
    let user_active = if json["user"]["active"].as_bool().unwrap() {
        "true"
    } else {
        "false"
    };

    // Create nested user context
    context.insert("user.name", VariableTy::String.with_data(user_name));
    context.insert("user.age", VariableTy::String.with_data(user_age));
    context.insert("user.active", VariableTy::Boolean.with_data(user_active));

    // Boolean flags
    let show_details = if json["show_details"].as_bool().unwrap() {
        "true"
    } else {
        "false"
    };
    let has_access = if json["has_access"].as_bool().unwrap() {
        "true"
    } else {
        "false"
    };
    context.insert("show_details", VariableTy::Boolean.with_data(show_details));
    context.insert("has_access", VariableTy::Boolean.with_data(has_access));

    // Items as an iterable
    let items = json["items"].as_array().unwrap();
    let mut items_data = String::new();

    for (i, item) in items.iter().enumerate() {
        let name = item["name"].as_str().unwrap();
        let value = item["value"].as_i64().unwrap().to_string();
        let special = if item["special"].as_bool().unwrap() {
            "true"
        } else {
            "false"
        };

        context.insert("item.name", VariableTy::String.with_data(name.to_owned()));
        context.insert("item.value", VariableTy::String.with_data(value));
        context.insert("item.special", VariableTy::Boolean.with_data(special));

        if i > 0 {
            items_data.push_str(", ");
        }
        items_data.push_str("item");
    }

    context.insert("items", VariableTy::Iterable.with_data(items_data));

    context
}

criterion_group!(benches, minilate_benchmark);
criterion_main!(benches);
