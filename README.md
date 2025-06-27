# Imprintor

A fast and efficient Elixir library for generating PDF documents from [Typst](https://typst.app/) templates using native Rust implementations.

## Features

- 🚀 **Fast PDF generation** using native Rust and Typst
- 📄 **Template-based** document creation with data integration
- 🔧 **Simple API** for Elixir developers
- 💾 **In-memory PDF generation** - returns PDF binary data
- 📝 **Typst syntax** support for beautiful document formatting
- 🧩 **Rich data support** - pass complex nested data structures
- 🔄 **Data iteration** - loop through arrays and nested objects in templates
- 🎨 **Custom fonts** - support for additional font files

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
# Define a Typst template with data access
template = """
= Hello #elixir_data.name!

*Date:* #elixir_data.date

This is a sample document generated using Imprintor.

== Details

Here are some details:
- *Name:* #elixir_data.name
- *Email:* #elixir_data.email
- *Age:* #elixir_data.age

Thanks for using Imprintor!
"""

# Provide data to fill the template
data = %{
  "name" => "John Doe",
  "date" => "2024-01-15",
  "email" => "john@example.com",
  "age" => 30
}

# Create a configuration and generate PDF
config = Imprintor.Config.new(template, data)
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)

# Save to file
File.write!("output.pdf", pdf_binary)
```

### Working with Complex Data and Iteration

```elixir
# Template with data iteration using elixir_data
template = """
= Employee Directory

#for employee in elixir_data.employees [
  == #employee.name

  *Position:* #employee.position
  *Department:* #employee.department
  *Email:* #employee.email

  ---
]
"""

# Complex nested data structure
data = %{
  "employees" => [
    %{
      "name" => "Alice Johnson",
      "position" => "Software Engineer", 
      "department" => "Engineering",
      "email" => "alice@company.com"
    },
    %{
      "name" => "Bob Smith",
      "position" => "Product Manager",
      "department" => "Product", 
      "email" => "bob@company.com"
    }
  ]
}

config = Imprintor.Config.new(template, data)
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)
File.write!("employee_directory.pdf", pdf_binary)
```

## Template Syntax and Data Access

Imprintor uses Typst syntax for document formatting. Data is made available under the `elixir_data` variable:

- `#elixir_data.field_name` - Access simple fields
- `#elixir_data.nested.field` - Access nested object fields  
- `#for item in elixir_data.array [ ... ]` - Iterate over arrays
- `#item.field` - Access fields within iteration loops

### Data Access Example

```elixir
data = %{
  "user" => %{
    "name" => "John Doe",
    "contact" => %{
      "email" => "john@example.com",
      "phone" => "+1-555-1234"
    }
  },
  "orders" => [
    %{"id" => 1, "product" => "Widget A", "price" => 29.99},
    %{"id" => 2, "product" => "Widget B", "price" => 39.99}
  ]
}

template = """
= Customer: #elixir_data.user.name

*Contact:*
- Email: #elixir_data.user.contact.email
- Phone: #elixir_data.user.contact.phone

== Recent Orders
#for order in elixir_data.orders [
  - #order.product: \\$#order.price
]
"""

config = Imprintor.Config.new(template, data)
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)
```

### Common Typst Formatting

- `= Heading 1` - Main heading
- `== Heading 2` - Subheading  
- `=== Heading 3` - Sub-subheading
- `*bold text*` - Bold text
- `_italic text_` - Italic text
- `- Item 1` - Bullet list
- `1. Item 1` - Numbered list

### Variable Substitution Syntax

Imprintor uses Typst syntax for data access:

```typst
= Customer Report

*Name:* #elixir_data.customer.name
*Email:* #elixir_data.customer.email

== Orders
#for order in elixir_data.orders [
  - #order.product: \\$#order.total
]
```

## API Reference

### `Imprintor.compile_to_pdf/1`

Compiles a Typst template with data and returns a PDF binary.

- `config` - An `Imprintor.Config` struct containing the template and data
- Returns: `{:ok, pdf_binary}` or `{:error, reason}`

Example:
```elixir
config = Imprintor.Config.new(template, data)
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)
```

### `Imprintor.Config.new/3`

Creates a new configuration for PDF compilation.

- `source_document` - String containing the Typst template
- `data` - Map of key-value pairs accessible via `elixir_data` in templates (optional, defaults to `%{}`)
- `opts` - Keyword list of options (optional, defaults to `[]`)
  - `:extra_fonts` - List of additional font paths
  - `:root_directory` - Root directory for relative file paths (defaults to `"."`)
- Returns: `%Imprintor.Config{}` struct

Example:
```elixir
config = Imprintor.Config.new(
  template, 
  %{"name" => "John"}, 
  extra_fonts: ["/path/to/font.ttf"],
  root_directory: "/project/templates"
)
```

## Examples

### Basic Example

```elixir
# Simple template with data access
template = "= Hello #elixir_data.name\n\nThis is a test document."
data = %{"name" => "World"}

config = Imprintor.Config.new(template, data)
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)
File.write!("hello.pdf", pdf_binary)
```

### List Iteration Example

```elixir
template = """
= Simple List Test

#for item in elixir_data.items [
  - #item
]
"""

data = %{"items" => ["Apple", "Banana", "Cherry"]}
config = Imprintor.Config.new(template, data)
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)
```

### Complex Data Example

```elixir
template = """
= Sales Report

#for region in elixir_data.regions [
  == #region.name Region

  *Total Sales:* \\$#region.total_sales

  === Products
  #for product in region.products [
    - #product.name: \\$#product.revenue
  ]

  ---
]
"""

data = %{
  "regions" => [
    %{
      "name" => "North",
      "total_sales" => 125_000,
      "products" => [
        %{"name" => "Widget A", "revenue" => 45000},
        %{"name" => "Widget B", "revenue" => 35000}
      ]
    },
    %{
      "name" => "South", 
      "total_sales" => 98000,
      "products" => [
        %{"name" => "Widget C", "revenue" => 55000},
        %{"name" => "Widget D", "revenue" => 43000}
      ]
    }
  ]
}

config = Imprintor.Config.new(template, data)
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)
```

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
- Basic Typst template compilation using native Rust
- PDF generation with data access using `elixir_data` variable  
- Support for nested data structures and array iteration
- Configuration-based API with `Imprintor.Config`
- Custom fonts support
- Comprehensive test suite

