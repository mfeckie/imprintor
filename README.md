# Imprintor

A fast and efficient Elixir library for generating PDF documents from [Typst](https://typst.app/) templates using native Rust implementations.

## Features

- 🚀 **Fast PDF generation** using native Rust and Typst
- 📄 **Template-based** document creation with variable substitution
- 🔧 **Simple API** for Elixir developers
- 💾 **Flexible output** - generate PDFs in memory or save to files
- 📝 **Typst syntax** support for beautiful document formatting

## Installation

Add `imprintor` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:imprintor, "~> 0.1.0"}
  ]
end
```

## Usage

### Basic PDF Generation

```elixir
# Define a Typst template with variables
template = """
= Hello {{name}}!

*Date:* {{date}}

This is a sample document generated using Imprintor.

== Details

Here are some details:
- *Name:* {{name}}
- *Email:* {{email}}
- *Age:* {{age}}

Thanks for using Imprintor!
"""

# Provide data to fill the template
data = %{
  "name" => "John Doe",
  "date" => "2024-01-15",
  "email" => "john@example.com",
  "age" => "30"
}

# Generate PDF
{:ok, pdf_binary} = Imprintor.compile_to_pdf(template, data)

# Save to file
File.write!("output.pdf", pdf_binary)
```

### Save PDF Directly to File

```elixir
template = "= My Document\n\nThis will be saved directly to a file."
data = %{}

:ok = Imprintor.compile_to_file(template, data, "my_document.pdf")
```

### Load Template from File

Create a template file (`templates/invoice.typ`):

```typst
= Invoice #{{invoice_number}}

*Date:* {{date}}
*To:* {{client_name}}
*From:* {{company_name}}

== Items

{{item_description}}

*Total:* ${{total}}

Thank you for your business!
```

Then use it in your Elixir code:

```elixir
data = %{
  "invoice_number" => "12345",
  "date" => "2024-01-15",
  "client_name" => "Acme Corp",
  "company_name" => "My Business",
  "item_description" => "Web development services",
  "total" => "2,500.00"
}

{:ok, pdf_binary} = Imprintor.compile_from_file("templates/invoice.typ", data)
File.write!("invoice_12345.pdf", pdf_binary)
```

## Template Syntax

Imprintor uses Typst syntax for document formatting. Variables are substituted using `{{variable_name}}` syntax.

### Common Typst Formatting

- `= Heading 1` - Main heading
- `== Heading 2` - Subheading  
- `=== Heading 3` - Sub-subheading
- `*bold text*` - Bold text
- `_italic text_` - Italic text
- `- Item 1` - Bullet list
- `1. Item 1` - Numbered list

### Variable Substitution

Variables in templates should be wrapped in double curly braces:

```typst
= Welcome {{user_name}}!

Your email is {{email}} and you joined on {{join_date}}.
```

All variable values are converted to strings before substitution.

## API Reference

### `Imprintor.compile_to_pdf/2`

Compiles a Typst template with data and returns a PDF binary.

- `template` - String containing the Typst template
- `data` - Map of key-value pairs for variable substitution
- Returns: `{:ok, pdf_binary}` or `{:error, reason}`

### `Imprintor.compile_to_file/3`

Compiles a template and saves the PDF to a file.

- `template` - String containing the Typst template
- `data` - Map of key-value pairs for variable substitution
- `output_path` - File path where the PDF should be saved
- Returns: `:ok` or `{:error, reason}`

### `Imprintor.compile_from_file/2`

Loads a template from a file and compiles it with data.

- `template_path` - Path to the Typst template file
- `data` - Map of key-value pairs for variable substitution
- Returns: `{:ok, pdf_binary}` or `{:error, reason}`

## Examples

Check out the `ImprintorExamples` module for more examples:

```elixir
# Run basic examples
ImprintorExamples.basic_example()
ImprintorExamples.template_file_example()
ImprintorExamples.save_to_file_example()
```

## Requirements

- Elixir 1.17+
- Rust (for compilation)
- System fonts (automatically detected)

## Performance

Imprintor uses native Rust implementations for PDF compilation, making it significantly faster than pure Elixir solutions. The library is suitable for high-throughput document generation.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## Changelog

### 0.1.0

- Initial release
- Basic Typst template compilation
- PDF generation with variable substitution
- File I/O operations
- Comprehensive test suite

