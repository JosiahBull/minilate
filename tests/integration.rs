mod fixtures;

use fixtures::get_engine;
use minilate::{Context, MinilateEngine, Variable, VariableTy};

#[test]
fn test_basic_substitution() {
    let mut engine = get_engine();
    engine
        .add_template("Template A", "Hello, {{ name }}!")
        .unwrap();

    let mut context = engine.context("Template A", &Default::default());
    assert_eq!(context.len(), 1, "should have a single element");
    let (name, type_) = context.pop().unwrap();
    assert_eq!(name, "name");
    assert_eq!(
        type_,
        VariableTy::String,
        "Expected variable to be a string-like"
    );

    let context = Context::new()
        .insert("name", type_.with_data("Jessica"))
        .to_owned();

    let rendered = engine.render("Template A", Some(&context)).unwrap();

    assert_eq!(
        rendered, "Hello, Jessica!",
        "Rendered string should match the template."
    );
}

#[test]
fn test_basic_iteration() {
    let mut engine = get_engine();
    engine
        .add_template(
            "Template A",
            "{{% for cat in cats %}}Greetings {{ cat }}\n{{% end for %}}",
        )
        .unwrap();

    let mut context = engine.context("Template A", &Default::default());
    assert_eq!(context.len(), 1, "should have a single element");
    let (name, type_) = context.pop().unwrap();
    assert_eq!(name, "cats");
    assert_eq!(
        type_,
        VariableTy::Iterable,
        "Expected variable to be an iterable-like"
    );
}

#[test]
fn test_non_iterable_passed_to_loop() {}
