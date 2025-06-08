# Minilate

[![Crates.io](https://img.shields.io/crates/v/minilate.svg)](https://crates.io/crates/minilate)
[![Documentation](https://docs.rs/minilate/badge.svg)](https://docs.rs/minilate)
[![Tests](https://github.com/JosiahBull/minilate/workflows/Tests/badge.svg)](https://github.com/JosiahBull/minilate/actions)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)

A templating engine prioritising minimal binary size and speed at the cost of feature set.

## 🚀 Quick Start

```rust
use minilate::{MinilateEngine, Context, VariableTy, MinilateInterface};

// Create engine and add template
let mut engine = MinilateEngine::new();
engine.add_template("hello", "Hello, {{ name }}!").unwrap();

// Create context
let mut context = Context::new();
context.insert("name", VariableTy::String.with_data("World".to_string()));

// Render
let result = engine.render("hello", Some(&context)).unwrap();
assert_eq!(result, "Hello, World!");
```

## ⚡ Performance

Minilate is designed when only need a a core feature set and care deeply about performance and binary size.

| Engine     | Binary Size   | Rel. Size   | Time/Template | Rel. Perf.  |
|------------|---------------|-------------|---------------|-------------|
| Minilate   | 1.56 MB       | 1.00x       | 205.08 µs     | 1.00x       |
| Handlebars | 1.75 MB       | 1.12x       | 776.79 µs     | 3.79x       |
| MiniJinja  | 2.00 MB       | 1.28x       | 438.03 µs     | 2.154       |

> 📊 Minilate achieves **12-28% smaller** binaries while being **~2-4x faster** than alternatives.

*Benchmarks: 100 complex templates × 50 iterations each. Template includes conditionals, loops, and nested data. See [Benchmarking](#-benchmarking) for details.*

## 🚀 Features

- **Simple replacements** using `{{ variable }}`
- **Conditional blocks** with boolean logic using `{{% if <condition> %}}`, `{{% else if %}}`, and `{{% else %}}`
  - NOT: `!`
  - AND: `&&`
  - OR: `||`
- **For loops** with `{{% for var in iterable %}}`
- **Nested template injection** using `{{<< <template_file_name>.tmpl }}`
- **Escaping** with `\{{` or `\{{%`

## 🛠️ Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
minilate = "0.1"
```

## 📖 Examples

### Basic Variable Substitution
```tmpl
Welcome to {{ site_name }}!
Your username is {{ user.name }}.
```

### Conditional Logic
```tmpl
{{% if user && is_active %}}
  Hello, {{ user }}! You have {{ message_count }} new messages.
{{% elif !is_active %}}
  Your account is inactive. Please contact support.
{{% else %}}
  Hello, Guest! Please log in to continue.
{{% endif %}}
```

### Loops and Nested Data
```tmpl
# Shopping Cart
{{% if items %}}
Total items: {{ item_count }}

{{% for item in items %}}
- {{ item.name }} ({{ item.quantity }}x) - ${{ item.price }}
  {{% if item.special %}}**ON SALE!**{{% endif %}}
{{% endfor %}}

Total: ${{ total_price }}
{{% else %}}
Your cart is empty.
{{% endif %}}
```

### Template Inclusion
```tmpl
{{<< header.tmpl }}

# Main Content
{{ content }}

{{<< footer.tmpl }}
```

### Complex Example
```tmpl
# User Profile: {{ user.name }}

{{% if user.active %}}
**Status:** Active User (Age: {{ user.age }})

{{% if show_details && has_access %}}
## Recent Activity
{{% for activity in recent_activities %}}
- {{ activity.date }}: {{ activity.description }}
  {{% if activity.important %}}⚠️ **Important**{{% endif %}}
{{% endfor %}}
{{% elif !has_access %}}
*You don't have permission to view activity details*
{{% endif %}}
{{% else %}}
**Status:** Inactive User
{{% endif %}}
```

## 📊 Benchmarking

To run benchmarks locally:

```bash
./scripts/benchmark.sh
```

This script:
- Builds all benchmark binaries
- Runs performance tests against Minilate, Handlebars, and MiniJinja
- Analyzes binary sizes
- Compares results with previous runs
- Saves results for historical tracking

The benchmark tests template rendering with:
- 100 different contexts
- 50 iterations per engine
- Complex templates with conditionals, loops, and nested data

## 📄 License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
