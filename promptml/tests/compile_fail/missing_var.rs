// This file is used with `trybuild` to verify the runtime error message when a
// required variable is not supplied.  The `prompt!` macro validates template
// syntax at compile time; the missing variable is detected at runtime by
// `build()` returning `Err(PromptError::MissingVariable("name"))`.
//
// To activate this test, add a trybuild runner in integration tests:
//
//   #[test]
//   fn compile_fail_tests() {
//       let t = trybuild::TestCases::new();
//       t.compile_fail("tests/compile_fail/*.rs");
//   }

fn main() {
    let tmpl = promptml::prompt!("Hello {name}");
    // Intentionally omitting `.set("name", …)` — `build()` will return Err.
    let _ = tmpl.render().build();
}
