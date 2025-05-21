# Minilate

## Features

Minilate is a templating engine priotising a minimal binary size at the cost of runtime speed and featureset.

- Simple replacments using `{{ variable }}`.
- If blocks with boolean logic using `{{% if <condition> %}}`, and `{{% else if %}}` or `{{% else %}}`.
  - NOT: `!`
  - AND: `&&`
  - OR: `||`
- For loops with `{{% for var in iterable %}}`
- Nested Template injection using `{{<< <template_file_name>.tmpl }}`
- Escaping with `\{{` or `\{{%`

### Examples

```tmpl
{{ title }}
{{% if user && is_active %}}
  Hello, {{ user }}!
{{% elif !is_active %}}
  Your account is inactive.
{{% else %}}
  Hello, Guest!
{{% endif %}}
{{% for item in items %}}
  - {{ item }}
{{% endfor %}}
```

Please find more examples in the /examples directory.
