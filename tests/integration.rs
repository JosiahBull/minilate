mod fixtures;

use fixtures::{generate_random_whitespace, generate_random_whitespace_at_least_one, get_engine};
use minilate::{Context, MinilateError, MinilateInterface, VariableTy};

#[test]
#[ntest::timeout(100)]
fn test_basic_substitution() {
    let mut engine = get_engine();
    engine
        .add_template("Template A", "Hello, {{ name }}!")
        .unwrap();

    let context = engine.context("Template A", &Default::default());
    assert_eq!(context.len(), 1, "should have a single element");

    // Check the only element in context
    let (name, type_) = context.first().unwrap();
    assert_eq!(*name, "name");
    assert_eq!(
        *type_,
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
#[ntest::timeout(100)]
fn test_basic_iteration() {
    let template = format!(
        "{{{{%{}for{}cat{}in{}cats{}%}}}}Greetings {{{{{}cat{}}}}}\n{{{{%{}endfor{}%}}}}",
        generate_random_whitespace(),
        generate_random_whitespace_at_least_one(),
        generate_random_whitespace_at_least_one(),
        generate_random_whitespace_at_least_one(),
        generate_random_whitespace(),
        generate_random_whitespace(),
        generate_random_whitespace(),
        generate_random_whitespace(),
        generate_random_whitespace(),
    );

    dbg!(&template);

    let mut engine = get_engine();
    engine.add_template("Template A", template).unwrap();

    let context = engine.context("Template A", &Default::default());
    assert_eq!(
        context.len(),
        2,
        "should have two elements: 'cats' and 'cat'"
    );

    // Filter to only check the 'cats' variable
    let cats_var = context.iter().find(|(name, _)| *name == "cats").unwrap();
    assert_eq!(cats_var.0, "cats");
    assert_eq!(
        cats_var.1,
        VariableTy::Iterable,
        "Expected variable to be an iterable-like"
    );

    // Now let's test the actual rendering
    let context = Context::new()
        .insert(
            "cats",
            VariableTy::Iterable.with_data("Fluffy, Whiskers, Mittens"),
        )
        .to_owned();

    let rendered = engine.render("Template A", Some(&context)).unwrap();
    let expected = "Greetings Fluffy\nGreetings Whiskers\nGreetings Mittens\n";
    assert_eq!(rendered, expected);
}

#[test]
#[ntest::timeout(100)]
fn test_non_iterable_passed_to_loop() {
    let mut engine = get_engine();
    engine
        .add_template(
            "Loop Template",
            "{{% for item in items %}}{{ item }}{{% endfor %}}",
        )
        .unwrap();

    let context = Context::new()
        .insert("items", VariableTy::String.with_data("Not an iterable"))
        .to_owned();

    let result = engine.render("Loop Template", Some(&context));

    match result {
        Err(MinilateError::TypeMismatch {
            variable_name,
            expected,
            found,
        }) => {
            assert_eq!(variable_name, "items");
            assert_eq!(expected, VariableTy::Iterable);
            assert_eq!(found, VariableTy::String);
        }
        _ => panic!("Expected a TypeMismatch error, got {:?}", result),
    }
}

#[test]
#[ntest::timeout(100)]
fn test_if_statement() {
    let mut engine = get_engine();
    engine
        .add_template(
            "Conditional",
            "Hello{{% if show_name %}}, {{ name }}{{% endif %}}!",
        )
        .unwrap();

    // Test with show_name = true
    let context = Context::new()
        .insert("show_name", VariableTy::Boolean.with_data("true"))
        .insert("name", VariableTy::String.with_data("World"))
        .to_owned();

    let rendered = engine.render("Conditional", Some(&context)).unwrap();
    assert_eq!(rendered, "Hello, World!");

    // Test with show_name = false
    let context = Context::new()
        .insert("show_name", VariableTy::Boolean.with_data("false"))
        .insert("name", VariableTy::String.with_data("World"))
        .to_owned();

    let rendered = engine.render("Conditional", Some(&context)).unwrap();
    assert_eq!(rendered, "Hello!");
}

#[test]
#[ntest::timeout(100)]
fn test_if_else_statement() {
    let template = format!(
        "{{{{%{}if{}condition{}%}}}}True{{{{%{}else{}%}}}}False{{{{%{}endif{}%}}}}",
        generate_random_whitespace(),
        generate_random_whitespace_at_least_one(),
        generate_random_whitespace(),
        generate_random_whitespace(),
        generate_random_whitespace(),
        generate_random_whitespace(),
        generate_random_whitespace(),
    );

    dbg!(&template);

    let mut engine = get_engine();
    engine.add_template("IfElse", template).unwrap();

    // Test with condition = true
    let context = Context::new()
        .insert("condition", VariableTy::Boolean.with_data("true"))
        .to_owned();

    let rendered = engine.render("IfElse", Some(&context)).unwrap();
    assert_eq!(rendered, "True");

    // Test with condition = false
    let context = Context::new()
        .insert("condition", VariableTy::Boolean.with_data("false"))
        .to_owned();

    let rendered = engine.render("IfElse", Some(&context)).unwrap();
    assert_eq!(rendered, "False");
}

#[test]
#[ntest::timeout(100)]
fn test_logical_operators() {
    let mut engine = get_engine();
    engine
        .add_template(
            "Logic",
            "{{% if a %}}{{%if b %}}Both true{{% else %}}One true{{% endif %}}{{% else %}}{{% if b %}}One true{{% else %}}None true{{% endif %}}{{% endif %}}",
        )
        .unwrap();

    // Test with both true
    let context = Context::new()
        .insert("a", VariableTy::Boolean.with_data("true"))
        .insert("b", VariableTy::Boolean.with_data("true"))
        .to_owned();

    let rendered = engine.render("Logic", Some(&context)).unwrap();
    assert_eq!(rendered, "Both true");

    // Test with only 'a' true
    let context = Context::new()
        .insert("a", VariableTy::Boolean.with_data("true"))
        .insert("b", VariableTy::Boolean.with_data("false"))
        .to_owned();

    let rendered = engine.render("Logic", Some(&context)).unwrap();
    assert_eq!(rendered, "One true");

    // Test with both false
    let context = Context::new()
        .insert("a", VariableTy::Boolean.with_data("false"))
        .insert("b", VariableTy::Boolean.with_data("false"))
        .to_owned();

    let rendered = engine.render("Logic", Some(&context)).unwrap();
    assert_eq!(rendered, "None true");
}

#[test]
#[ntest::timeout(100)]
fn test_missing_variable() {
    let mut engine = get_engine();
    engine
        .add_template("Missing", "Hello, {{ name }}!")
        .unwrap();

    let empty_context = Context::new();

    // Variable is missing from context
    let result = engine.render("Missing", Some(&empty_context));
    assert!(matches!(result, Err(MinilateError::MissingVariable { .. })));
}

#[test]
#[ntest::timeout(100)]
fn test_nested_structures() {
    let mut engine = get_engine();
    engine
        .add_template(
            "Nested",
            "{{% for person in people %}}{{% if person %}}{{ person }}{{% else %}}Anonymous{{% endif %}}, {{% endfor %}}",
        )
        .unwrap();

    let context = Context::new()
        .insert("people", VariableTy::Iterable.with_data("Alice, , Bob"))
        .to_owned();

    let rendered = engine.render("Nested", Some(&context)).unwrap();
    assert_eq!(rendered, "Alice, Anonymous, Bob, ");
}

#[test]
#[ntest::timeout(100)]
fn test_complex_nested_structures() {
    let mut engine = get_engine();
    engine
        .add_template(
            "ComplexNested",
            "{{% for continent in continents %}}\
                Continent: {{ continent }}\n\
                {{% if has_countries %}}\
                    {{% for country in countries %}}\
                        - {{ country }}\n\
                    {{% endfor %}}\
                {{% else %}}\
                    No countries defined.\n\
                {{% endif %}}\
             {{% endfor %}}",
        )
        .unwrap();

    let context = Context::new()
        .insert("continents", VariableTy::Iterable.with_data("Europe, Asia"))
        .insert("has_countries", VariableTy::Boolean.with_data("true"))
        .insert(
            "countries",
            VariableTy::Iterable.with_data("France, Germany, Japan, China"),
        )
        .to_owned();

    let rendered = engine.render("ComplexNested", Some(&context)).unwrap();

    // Each continent should list all countries (simplified implementation)
    assert!(rendered.contains("Continent: Europe"));
    assert!(rendered.contains("Continent: Asia"));
    assert!(rendered.contains("- France"));
    assert!(rendered.contains("- Japan"));
}

#[test]
#[ntest::timeout(100)]
fn test_boolean_conditions() {
    let mut engine = get_engine();
    engine
        .add_template(
            "BooleanConditions",
            "{{% if flag1 %}}Condition 1{{% else %}}Condition 2{{% endif %}}",
        )
        .unwrap();

    // Test case 1: flag1=true -> Condition 1
    let context = Context::new()
        .insert("flag1", VariableTy::Boolean.with_data("true"))
        .to_owned();
    assert_eq!(
        engine.render("BooleanConditions", Some(&context)).unwrap(),
        "Condition 1"
    );

    // Test case 2: flag1=false -> Condition 2
    let context = Context::new()
        .insert("flag1", VariableTy::Boolean.with_data("false"))
        .to_owned();
    assert_eq!(
        engine.render("BooleanConditions", Some(&context)).unwrap(),
        "Condition 2"
    );
}

#[test]
#[ntest::timeout(100)]
fn test_variable_data_missing() {
    let mut engine = get_engine();
    engine
        .add_template("MissingData", "Hello, {{ name }}!")
        .unwrap();

    // Create a context with a variable with no data
    let mut context = Context::new();
    context.insert("name", VariableTy::String.with_data(""));

    let result = engine.render("MissingData", Some(&context));
    assert!(matches!(
        result,
        Err(MinilateError::MissingVariableData { .. })
    ));
}

#[test]
#[ntest::timeout(100)]
fn test_duplicate_template() {
    let mut engine = get_engine();

    // Add first template
    engine.add_template("Duplicate", "First version").unwrap();

    // Try to add with same name
    let result = engine.add_template("Duplicate", "Second version");
    assert!(matches!(result, Err(MinilateError::TemplateExists { .. })));
}

#[test]
#[ntest::timeout(100)]
fn test_render_missing_template() {
    let engine = get_engine();

    let result = engine.render("NonExistentTemplate", None);
    assert!(matches!(result, Err(MinilateError::MissingTemplate { .. })));
}

#[test]
#[ntest::timeout(100)]
fn test_context_collector() {
    let mut engine = get_engine();
    engine
        .add_template(
            "ContextTest",
            "{{ var1 }} {{% if condition %}}{{ var2 }}{{% else %}}{{ var3 }}{{% endif %}} \
             {{% for item in items %}}{{ item }} {{ var4 }}{{% endfor %}}",
        )
        .unwrap();

    let context = Context::new();
    let variables = engine.context("ContextTest", &context);

    // Should find var1, condition, var2, var3, items, var4, and item (from the loop)
    assert_eq!(variables.len(), 7);

    // Check specific variables
    assert!(
        variables
            .iter()
            .any(|(name, ty)| *name == "var1" && *ty == VariableTy::String)
    );
    assert!(
        variables
            .iter()
            .any(|(name, ty)| *name == "condition" && *ty == VariableTy::Boolean)
    );
    assert!(
        variables
            .iter()
            .any(|(name, ty)| *name == "var2" && *ty == VariableTy::String)
    );
    assert!(
        variables
            .iter()
            .any(|(name, ty)| *name == "var3" && *ty == VariableTy::String)
    );
    assert!(
        variables
            .iter()
            .any(|(name, ty)| *name == "items" && *ty == VariableTy::Iterable)
    );
    assert!(
        variables
            .iter()
            .any(|(name, ty)| *name == "var4" && *ty == VariableTy::String)
    );
}

#[test]
#[ntest::timeout(100)]
fn test_whitespace_handling() {
    let mut engine = get_engine();
    engine
        .add_template(
            "Whitespace",
            "  {{   var1   }}  \n {{% if  condition  %}} \n Content \n {{% endif %}}",
        )
        .unwrap();

    let context = Context::new()
        .insert("var1", VariableTy::String.with_data("Value"))
        .insert("condition", VariableTy::Boolean.with_data("true"))
        .to_owned();

    let rendered = engine.render("Whitespace", Some(&context)).unwrap();
    assert_eq!(rendered, "  Value  \n  \n Content \n ");
}

#[test]
#[ntest::timeout(100)]
fn test_basic_template_referencing() {
    let mut engine = get_engine();

    engine
        .add_template("greeting", "Hello {{ name }}!")
        .unwrap();
    engine
        .add_template("welcome", "Welcome: {{<< greeting }}")
        .unwrap();

    // Check what variables the template needs
    let mut context = engine.context("welcome", &Default::default());
    assert_eq!(context.len(), 1);
    let (name, type_) = context.pop().unwrap();
    assert_eq!(name, "name");
    assert_eq!(type_, VariableTy::String);

    // Render and validate output
    let context = Context::new()
        .insert("name", VariableTy::String.with_data("John"))
        .to_owned();
    let rendered = engine.render("welcome", Some(&context)).unwrap();
    assert_eq!(rendered, "Welcome: Hello John!");
}

#[test]
#[ntest::timeout(100)]
fn test_conditional_template_referencing_if() {
    let mut engine = get_engine();

    engine
        .add_template("greeting", "Hello {{ name }}!")
        .unwrap();
    engine
        .add_template(
            "form",
            "Greetings{{% if polite %}} {{<< greeting }}{{% endif %}}.",
        )
        .unwrap();

    // When polite is not set, we just need to know if it exists
    let mut context = engine.context("form", &Default::default());
    assert_eq!(context.len(), 1);
    let (name, type_) = context.pop().unwrap();
    assert_eq!(name, "polite");
    assert_eq!(type_, VariableTy::Boolean);

    // When polite=true, we also need the referenced template's variables
    let mut context = engine.context(
        "form",
        Context::new().insert("polite", VariableTy::Boolean.with_data("true")),
    );
    assert_eq!(context.len(), 1);
    let (name, type_) = context.pop().unwrap();
    assert_eq!(name, "name");
    assert_eq!(type_, VariableTy::String);

    // Render with polite=false
    let context = Context::new()
        .insert("polite", VariableTy::Boolean.with_data("false"))
        .to_owned();
    let rendered = engine.render("form", Some(&context)).unwrap();
    assert_eq!(rendered, "Greetings.");

    // Render with polite=true
    let context = Context::new()
        .insert("polite", VariableTy::Boolean.with_data("true"))
        .insert("name", VariableTy::String.with_data("Alice"))
        .to_owned();
    let rendered = engine.render("form", Some(&context)).unwrap();
    assert_eq!(rendered, "Greetings Hello Alice!.");
}

#[test]
#[ntest::timeout(100)]
fn test_conditional_template_referencing_for() {
    let mut engine = get_engine();

    engine
        .add_template("greeting", "Hello {{ name }}!")
        .unwrap();
    engine
        .add_template(
            "group_greeting",
            "The team:{{% for name in names %}} {{<< greeting }}{{% endfor %}}",
        )
        .unwrap();

    // Check context with empty members list
    let mut context = engine.context("group_greeting", &Default::default());
    println!("{:?}", context);
    assert_eq!(context.len(), 1);
    let (name, type_) = context.pop().unwrap();
    assert_eq!(name, "names");
    assert_eq!(type_, VariableTy::Iterable);

    // Check context with members list (needs name from greeting template)
    let context_with_members = Context::new()
        .insert("names", VariableTy::Iterable.with_data("Alice,Bob"))
        .to_owned();
    let vars = engine.context("group_greeting", &context_with_members);
    assert_eq!(vars.len(), 0);

    // Render with empty members list
    let context = Context::new()
        .insert("names", VariableTy::Iterable.with_data(""))
        .to_owned();
    let rendered = engine.render("group_greeting", Some(&context)).unwrap();
    assert_eq!(rendered, "The team:");

    // Render with populated members list
    let context = Context::new()
        .insert("names", VariableTy::Iterable.with_data("Alice,Bob"))
        .to_owned();
    let rendered = engine.render("group_greeting", Some(&context)).unwrap();
    assert_eq!(rendered, "The team: Hello Alice! Hello Bob!");
}

#[test]
#[ntest::timeout(100)]
fn test_rendering_with_include_nested() {
    let mut engine = get_engine();

    engine
        .add_template("greeting", "Hello {{ name }}!")
        .unwrap();
    engine
        .add_template(
            "group_greeting",
            "The team:\n{{% for name in names %}}{{<< greeting }}\n{{% endfor %}}{{<< greeting }}\nIs the team lead.",
        )
        .unwrap();

    let mut context = engine.context("group_greeting", &Default::default());
    context.sort();
    let (name, type_) = context.pop().unwrap();
    assert_eq!(name, "names");
    assert_eq!(type_, VariableTy::Iterable);

    let (name, type_) = context.pop().unwrap();
    assert_eq!(name, "name");
    assert_eq!(type_, VariableTy::String);

    let context = Context::new()
        .insert("names", VariableTy::Iterable.with_data("John,Sarah"))
        .insert("name", VariableTy::String.with_data("Patrick"))
        .to_owned();

    let rendered = engine.render("group_greeting", Some(&context)).unwrap();
    assert_eq!(
        rendered,
        "The team:\nHello John!\nHello Sarah!\nHello Patrick!\nIs the team lead."
    );
}
