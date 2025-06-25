defmodule Imprintor do
  @moduledoc """
  Imprintor is a library for generating PDF documents from Typst templates.

  It provides functions to compile Typst templates with data interpolation
  and generate PDF documents using a native Rust implementation.
  """

  use Rustler, otp_app: :imprintor, crate: "imprintor"

  @doc """
  Compiles a Typst template with the given data and returns a PDF as binary.

  ## Parameters

  - `template`: A string containing the Typst template content
  - `data`: A map of key-value pairs to substitute in the template

  ## Examples

      iex> template = "= Hello {{name}}\\n\\nThis is a document for {{name}}."
      iex> data = %{"name" => "World"}
      iex> {:ok, pdf_binary} = Imprintor.compile_to_pdf(template, data)
      iex> is_binary(pdf_binary)
      true

  ## Template Syntax

  Variables in the template should be wrapped in double curly braces: `{{variable_name}}`

  The template uses Typst syntax for formatting:
  - `= Title` for headings
  - `== Subtitle` for subheadings
  - `*bold*` for bold text
  - `_italic_` for italic text
  - And much more according to Typst documentation
  """
  def compile_to_pdf(template, data \\ %{}) when is_binary(template) and is_map(data) do
    # Convert all keys and values to strings for the NIF
    string_data =
      data
      |> Enum.map(fn {k, v} -> {to_string(k), to_string(v)} end)
      |> Enum.into(%{})

    case compile_typst_to_pdf(template, string_data) do
      {:ok, pdf_binary} -> {:ok, pdf_binary}
      {:error, reason} -> {:error, reason}
      pdf_binary when is_binary(pdf_binary) -> {:ok, pdf_binary}
      error -> {:error, error}
    end
  end

  @doc """
  Compiles a Typst template with the given data and writes the PDF to a file.

  ## Parameters

  - `template`: A string containing the Typst template content
  - `data`: A map of key-value pairs to substitute in the template  
  - `output_path`: Path where the PDF file should be written

  ## Examples

      iex> template = "= Hello {{name}}\\n\\nThis is a document for {{name}}."
      iex> data = %{"name" => "John Doe"}
      iex> Imprintor.compile_to_file(template, data, "/tmp/output.pdf")
      :ok
  """
  def compile_to_file(template, data, output_path) do
    case compile_to_pdf(template, data) do
      {:ok, pdf_binary} ->
        File.write(output_path, pdf_binary)

      {:error, reason} ->
        {:error, reason}
    end
  end

  @doc """
  Loads a template from a file, compiles it with the given data, and returns a PDF.

  ## Parameters

  - `template_path`: Path to the Typst template file
  - `data`: A map of key-value pairs to substitute in the template

  ## Examples

      iex> data = %{"title" => "Simple Invoice"}
      iex> File.write!("test_template.typ", "= {{title}}\\n\\nThis is a simple test.")
      iex> {:ok, pdf_binary} = Imprintor.compile_from_file("test_template.typ", data)
      iex> File.rm("test_template.typ")
      iex> is_binary(pdf_binary)
      true
  """
  def compile_from_file(template_path, data \\ %{}) do
    case File.read(template_path) do
      {:ok, template_content} ->
        compile_to_pdf(template_content, data)

      {:error, reason} ->
        {:error, {:file_error, reason}}
    end
  end

  # Private NIF function - called by compile_to_pdf/2
  def compile_typst_to_pdf(_template, _data), do: :erlang.nif_error(:nif_not_loaded)
end
