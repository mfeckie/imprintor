defmodule Imprintor.Config do
  @moduledoc """
  Configuration struct for the Imprintor document processing system.

  This module defines the configuration structure used throughout the Imprintor
  application to specify document processing parameters, including source documents,
  fonts, directories, and template data.

  ## Struct Fields

  * `:source_document` - The path or identifier of the source document to process
  * `:extra_fonts` - A list of additional font files or paths to include
  * `:root_directory` - The root directory for resolving relative paths (defaults to ".")
  * `:data` - A map containing template data and variables for document generation

  ## Examples

      # Basic configuration
      config = Imprintor.Config.new("template.html")

      # Configuration with template data
      config = Imprintor.Config.new("invoice.html", %{
        customer_name: "John Doe",
        amount: 1500.00
      })

      # Configuration with extra fonts and custom root directory
      config = Imprintor.Config.new("report.html", %{title: "Annual Report"}, [
        extra_fonts: ["fonts/custom.ttf", "fonts/bold.ttf"],
        root_directory: "/path/to/project"
      ])
  """

  defstruct [:source_document, :extra_fonts, :root_directory, :data]

  @doc """
  Creates a new configuration struct.

  ## Parameters

  * `source_document` - The source document path or identifier (required)
  * `data` - A map containing template data and variables (optional, defaults to `%{}`)
  * `opts` - A keyword list of additional options (optional)

  ## Options

  * `:extra_fonts` - List of additional font files to include (defaults to `[]`)
  * `:root_directory` - Root directory for resolving relative paths (defaults to `"."`)

  ## Returns

  Returns a `%Imprintor.Config{}` struct with the specified configuration.

  ## Examples

      # Minimal configuration
      config = Imprintor.Config.new("document.html")

      # With template data
      config = Imprintor.Config.new("invoice.html", %{
        invoice_number: "INV-001",
        customer: "Acme Corp",
        items: [
          %{name: "Widget", price: 25.00, quantity: 2},
          %{name: "Gadget", price: 15.00, quantity: 1}
        ]
      })

      # With all options
      config = Imprintor.Config.new(
        "report.html",
        %{title: "Q4 Report", year: 2024},
        extra_fonts: ["fonts/arial.ttf"],
        root_directory: "/projects/reports"
      )
  """
  def new(source_document, data \\ %{}, opts \\ []) do
    extra_fonts = Keyword.get(opts, :extra_fonts, [])

    root_directory = Keyword.get(opts, :root_directory, ".")

    %__MODULE__{
      source_document: source_document,
      extra_fonts: extra_fonts,
      root_directory: root_directory,
      data: data
    }
  end
end
