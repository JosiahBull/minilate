#[cfg(feature = "serde")]
mod serde_tests {
    use minilate::{Context, MinilateEngine, MinilateInterface, Template, VariableTy};
    use serde_json;

    #[test]
    fn test_variable_ty_serialization() {
        let ty = VariableTy::String;
        let serialized = serde_json::to_string(&ty).unwrap();
        assert_eq!(serialized, r#""String""#);

        let deserialized: VariableTy = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, ty);
    }

    #[test]
    fn test_variable_serialization() {
        let var = VariableTy::String.with_data("test data");
        let serialized = serde_json::to_string(&var).unwrap();
        assert_eq!(serialized, r#"{"ty":"String","data":"test data"}"#);

        let mut context = Context::new();
        context.insert("test", VariableTy::String.with_data("test data"));

        let context_serialized = serde_json::to_string(&context).unwrap();
        let context_deserialized: Context = serde_json::from_str(&context_serialized).unwrap();

        assert_eq!(
            context_deserialized.get("test").unwrap().ty(),
            VariableTy::String
        );
        assert_eq!(
            context_deserialized.get("test").unwrap().data(),
            Some("test data")
        );
    }

    #[test]
    fn test_context_serialization() {
        let mut context = Context::new();
        context.insert("name", VariableTy::String.with_data("John"));
        context.insert("active", VariableTy::Boolean.with_data("true"));
        context.insert("items", VariableTy::Iterable.with_data("one, two, three"));

        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: Context = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.get("name").unwrap().data(), Some("John"));
        assert_eq!(deserialized.get("active").unwrap().data(), Some("true"));
        assert_eq!(
            deserialized.get("items").unwrap().data(),
            Some("one, two, three")
        );
    }

    #[test]
    fn test_template_serialization() {
        let template = Template::new("Hello, {{ name }}!".to_string()).unwrap();

        // Serialize the template
        let serialized = serde_json::to_string(&template).unwrap();

        // Deserialize back to a template
        let deserialized: Template = serde_json::from_str(&serialized).unwrap();

        // Create contexts for testing
        let mut context = Context::new();
        context.insert("name", VariableTy::String.with_data("World"));

        // Both templates should render the same output
        let original_output = template.render(&context, None::<&MinilateEngine>).unwrap();
        let deserialized_output = deserialized.render(&context, None::<&MinilateEngine>).unwrap();

        assert_eq!(original_output, deserialized_output);
        assert_eq!(original_output, "Hello, World!");
    }

    #[test]
    fn test_engine_serialization() {
        let mut engine = MinilateEngine::new();

        // Add some templates
        engine
            .add_template("greeting", "Hello, {{ name }}!")
            .unwrap();
        engine
            .add_template(
                "list",
                "Items: {{% for item in items %}}{{ item }}, {{% endfor %}}",
            )
            .unwrap();

        // Serialize the engine
        let serialized = serde_json::to_string(&engine).unwrap();

        // Deserialize back to an engine
        let deserialized: MinilateEngine = serde_json::from_str(&serialized).unwrap();

        // Create contexts for testing
        let mut context1 = Context::new();
        context1.insert("name", VariableTy::String.with_data("World"));

        let mut context2 = Context::new();
        context2.insert("items", VariableTy::Iterable.with_data("a, b, c"));

        // Both engines should render the same outputs
        assert_eq!(
            engine.render("greeting", Some(&context1)).unwrap(),
            deserialized.render("greeting", Some(&context1)).unwrap()
        );

        assert_eq!(
            engine.render("list", Some(&context2)).unwrap(),
            deserialized.render("list", Some(&context2)).unwrap()
        );
    }
}
