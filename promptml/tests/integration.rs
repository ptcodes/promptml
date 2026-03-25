use std::collections::HashMap;

use pretty_assertions::assert_eq;
use promptml::{Example, PromptError, PromptTemplate, Role};

#[test]
fn basic_render_all_vars_provided() {
    let tmpl = PromptTemplate::new("Hello, {name}! You are {age} years old.").unwrap();
    let result = tmpl
        .render()
        .set("name", "Alice")
        .set("age", "30")
        .build()
        .unwrap();
    assert_eq!(result, "Hello, Alice! You are 30 years old.");
}

#[test]
fn missing_variable_returns_error() {
    let tmpl = PromptTemplate::new("Hello, {name}!").unwrap();
    let err = tmpl.render().build().unwrap_err();
    match err {
        PromptError::MissingVariable(v) => assert_eq!(v, "name"),
        other => panic!("expected MissingVariable, got {other:?}"),
    }
}

#[test]
fn if_block_included_when_var_present() {
    let tmpl = PromptTemplate::new("Hello{{#if name}}, {name}{{/if}}!").unwrap();
    let result = tmpl.render().set("name", "Bob").build().unwrap();
    assert_eq!(result, "Hello, Bob!");
}

#[test]
fn if_block_omitted_when_var_absent() {
    let tmpl = PromptTemplate::new("Hello{{#if name}}, {name}{{/if}}!").unwrap();
    let result = tmpl.render().build().unwrap();
    assert_eq!(result, "Hello!");
}

#[test]
fn examples_block_renders_each_example() {
    let tmpl = PromptTemplate::new("Examples:\n{{#examples}}Q: {q}\nA: {a}{{/examples}}").unwrap();

    let mut e1 = HashMap::new();
    e1.insert("q".to_string(), "What is 2+2?".to_string());
    e1.insert("a".to_string(), "4".to_string());

    let mut e2 = HashMap::new();
    e2.insert("q".to_string(), "Capital of France?".to_string());
    e2.insert("a".to_string(), "Paris".to_string());

    let result = tmpl
        .render()
        .examples(vec![Example { vars: e1 }, Example { vars: e2 }])
        .build()
        .unwrap();

    assert_eq!(
        result,
        "Examples:\nQ: What is 2+2?\nA: 4\nQ: Capital of France?\nA: Paris\n"
    );
}

#[test]
fn to_messages_returns_system_and_user() {
    let tmpl =
        PromptTemplate::new_with_system("Answer this: {question}", Some("You are {persona}."))
            .unwrap();

    let messages = tmpl
        .render()
        .set("question", "What is Rust?")
        .set("persona", "a helpful assistant")
        .to_messages()
        .unwrap();

    assert_eq!(messages.len(), 2);

    assert_eq!(messages[0].role, Role::System);
    assert_eq!(messages[0].content, "You are a helpful assistant.");

    assert_eq!(messages[1].role, Role::User);
    assert_eq!(messages[1].content, "Answer this: What is Rust?");
}

#[test]
fn from_file_loads_toml_template() {
    let tmpl = PromptTemplate::from_file("tests/fixtures/test.toml").unwrap();
    let result = tmpl
        .render()
        .set("topic", "Rust")
        .set("tone", "friendly")
        .build()
        .unwrap();
    assert_eq!(result, "Tell me about Rust in a friendly tone.");
}
