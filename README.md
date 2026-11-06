# Imprintor

A fast and efficient Elixir library for generating PDF and PNG documents from [Typst](https://typst.app/) templates using native Rust implementations.

## Features

- 🚀 **Fast PDF and PNG generation** using native Rust and Typst
- 📄 **Template-based** document creation with data integration
- 💾 **In-memory generation** - returns PDF/PNG binary data
- 🖼️ **PNG rendering** - compile to one image per page, with configurable PPI
- 📝 **Typst syntax** support for beautiful document formatting
- 🧩 **Rich data support** - pass complex nested data structures
- 📚 **PDF standard selection** - enforce standards like `a-3a`, `ua-1`, and `1.7`
- 📎 **Tagged bytes support** - pass binary data for Typst APIs like `pdf.attach`

## Installation

Add `imprintor` to your list of dependencies in `mix.exs`:

```elixir
def deps do
  [
    {:imprintor, "~> 0.7"}
  ]
end
```

Imprintor ships precompiled NIFs for these targets, so you don't need a Rust toolchain:

- macOS: `aarch64-apple-darwin` (Apple Silicon), `x86_64-apple-darwin` (Intel)
- Linux: `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-gnu`, `aarch64-unknown-linux-musl`, `arm-unknown-linux-gnueabihf`
- Windows: `x86_64-pc-windows-msvc`

### Building from source

On any other target, compiling Imprintor fails with:

```
** (RuntimeError) precompiled NIF is not available for this target: "<your target>".
```

You can also build from source on a supported target, for example to use your own Rust toolchain.

1. **Install Rust 1.92 or newer** ([rustup](https://rustup.rs)). Typst 0.15 needs at least 1.92; an older `rustc` fails part-way through the build with `rustc 1.91.1 is not supported by the following packages`. Check with `rustc --version`, and update with `rustup update stable`.

2. **Add Rustler to your own dependencies.** It is optional in Imprintor, so it isn't fetched otherwise:

   ```elixir
   def deps do
     [
       {:imprintor, "~> 0.7"},
       {:rustler, ">= 0.0.0", optional: true}
     ]
   end
   ```

   Without it, forcing the build fails with `Rustler dependency is needed to force the build`.

3. **Force the build**, with either the environment variable or config:

   ```console
   $ mix deps.get
   $ FORCE_COMPILE=1 mix compile
   ```

   ```elixir
   # config/config.exs
   config :rustler_precompiled, :force_build, imprintor: true
   ```

   Use `mix compile` (or `mix deps.compile`), not `mix deps.compile imprintor` on its own. The plain commands compile Rustler before Imprintor. Compiling only Imprintor before Rustler has been compiled fails with the same `Rustler dependency is needed` error.

   The setting is read when Imprintor itself compiles. To switch an Imprintor that is already compiled over to a source build, run `mix deps.compile imprintor --force` with the setting on. The config setting applies to every later build automatically. The environment variable doesn't, so set it on any build that compiles Imprintor again (a fresh `_build`, CI, a release image).

## Usage

### Basic PDF Generation

```elixir
# Define a Typst template with data access
template = """
= Hello #sys.inputs.elixir_data.name!

*Date:* #sys.inputs.elixir_data.date

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

### PNG Generation

`Imprintor.compile_to_png/1` renders a template to one PNG binary per page,
returned as a list in page order:

```elixir
template = "= Hello #sys.inputs.elixir_data.name!"
config = Imprintor.Config.new(template, %{"name" => "World"})

{:ok, [png_binary]} = Imprintor.compile_to_png(config)
File.write!("output.png", png_binary)
```

Pass `:ppi` to control the rendered resolution (defaults to `144.0`, matching
`typst-cli`):

```elixir
config = Imprintor.Config.new(template, %{"name" => "World"}, ppi: 288.0)
{:ok, [png_binary]} = Imprintor.compile_to_png(config)
```

`Imprintor.compile_to_png_file/2` writes the image(s) straight to disk. A
single-page document is written to the given path as-is; a multi-page
document gets the 1-based page number inserted before the extension:

```elixir
{:ok, ["report-1.png", "report-2.png"]} =
  Imprintor.compile_to_png_file(config, "report.png")
```

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
