# Imprintor

A fast and efficient Elixir library for generating PDF documents from [Typst](https://typst.app/) templates using native Rust implementations.

## Features

- 🚀 **Fast PDF generation** using native Rust and Typst
- 📄 **Template-based** document creation with data integration
- 💾 **In-memory PDF generation** - returns PDF binary data
- 📝 **Typst syntax** support for beautiful document formatting
- 🧩 **Rich data support** - pass complex nested data structures
- 📚 **PDF standard selection** - enforce standards like `a-3a`, `ua-1`, and `1.7`
- 📎 **Tagged bytes support** - pass binary data for Typst APIs like `pdf.attach`

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
- *Name:* #sys.inputs.elixir_data.name
- *Email:* #sys.inputs.elixir_data.email
- *Age:* #sys.inputs.elixir_data.age

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

#for employee in sys.inputs.elixir_data.employees [
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

### Variable Substitution Syntax

Imprintor uses Typst syntax for data access:

```typst
= Customer Report

*Name:* #sys.inputs.elixir_data.customer.name
*Email:* #sys.inputs.elixir_data.customer.email

== Orders
#for order in sys.inputs.elixir_data.orders [
  - #order.product: \\$#order.total
]
```

### PDF Standard Option

```elixir
template = """
#set document(date: datetime.today())
= Invoice
"""

config = Imprintor.Config.new(template, %{}, pdf_standard: "a-3a")
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)
```

Supported values are:
`1.4`, `1.5`, `1.6`, `1.7`, `2.0`, `a-1a`, `a-1b`, `a-2a`, `a-2b`, `a-2u`, `a-3a`, `a-3b`, `a-3u`, `a-4`, `a-4e`, `a-4f`, `ua-1`.

### Tagged Bytes for `pdf.attach`

```elixir
template = """
#set document(date: datetime.today())

#pdf.attach(
  "factur-x.xml",
  sys.inputs.elixir_data.factur_x_xml,
  relationship: "alternative",
  mime-type: "application/xml",
  description: "Factur-X XML",
)

= Invoice
"""

data = %{
  "factur_x_xml" => Imprintor.bytes("<xml>...</xml>")
}

config = Imprintor.Config.new(template, data, pdf_standard: "a-3a")
{:ok, pdf_binary} = Imprintor.compile_to_pdf(config)
```

`Imprintor.bytes/1` returns the tagged tuple `{:bytes, binary}`.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request
