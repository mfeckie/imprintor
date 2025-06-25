defmodule ImprintorTest do
  use ExUnit.Case

  test "compile_to_pdf with simple template" do
    template = "= Hello {{name}}\n\nThis is a test document."
    data = %{"name" => "World"}

    case Imprintor.compile_to_pdf(template, data) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0
        # PDF files start with "%PDF"
        assert String.starts_with?(pdf_binary, "%PDF")

      {:error, reason} ->
        flunk("Expected successful PDF generation, got error: #{inspect(reason)}")
    end
  end

  test "compile_to_pdf with empty data" do
    template = "= Simple Document\n\nThis document has no variables."

    case Imprintor.compile_to_pdf(template, %{}) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0

      {:error, reason} ->
        flunk("Expected successful PDF generation, got error: #{inspect(reason)}")
    end
  end

  test "compile_to_pdf with multiple variables" do
    template = """
    = {{title}}

    *Author:* {{author}}
    *Date:* {{date}}

    {{content}}
    """

    data = %{
      "title" => "Test Document",
      "author" => "Test Author",
      "date" => "2024-01-01",
      "content" => "This is the content of the document."
    }

    case Imprintor.compile_to_pdf(template, data) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0

      {:error, reason} ->
        flunk("Expected successful PDF generation, got error: #{inspect(reason)}")
    end
  end

  test "compile_to_file creates a file" do
    template = "= File Test\n\nThis document will be saved to a file."
    data = %{}
    output_path = "/tmp/test_output.pdf"

    # Clean up any existing file
    File.rm(output_path)

    case Imprintor.compile_to_file(template, data, output_path) do
      :ok ->
        assert File.exists?(output_path)
        {:ok, content} = File.read(output_path)
        assert String.starts_with?(content, "%PDF")
        # Clean up
        File.rm(output_path)

      {:error, reason} ->
        flunk("Expected successful file creation, got error: #{inspect(reason)}")
    end
  end

  test "compile_from_file with existing template" do
    # Create a temporary template file
    template_content = "= File Template Test\n\nHello {{name}}!"
    template_path = "/tmp/test_template.typ"
    File.write!(template_path, template_content)

    data = %{"name" => "Test User"}

    case Imprintor.compile_from_file(template_path, data) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0

      {:error, reason} ->
        flunk("Expected successful PDF generation from file, got error: #{inspect(reason)}")
    end

    # Clean up
    File.rm(template_path)
  end

  test "compile_from_file with non-existent file" do
    case Imprintor.compile_from_file("non_existent_file.typ", %{}) do
      {:error, {:file_error, :enoent}} ->
        # This is expected
        :ok

      other ->
        flunk("Expected file error, got: #{inspect(other)}")
    end
  end
end
