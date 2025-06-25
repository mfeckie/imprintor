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

  test "compile_to_pdf with simple list iteration" do
    template = """
    = Simple List Test

    #for item in json_data.items [
      - #item
    ]
    """

    data = %{
      "items" => ["Apple", "Banana", "Cherry"]
    }

    case Imprintor.compile_to_pdf(template, data) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0
        assert String.starts_with?(pdf_binary, "%PDF")

      {:error, reason} ->
        flunk(
          "Expected successful PDF generation with simple list, got error: #{inspect(reason)}"
        )
    end
  end

  test "compile_to_pdf with nested data structures and complex iteration" do
    template = """
    = Sales Report

    #for region in json_data.regions [
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

    case Imprintor.compile_to_pdf(template, data) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0
        assert String.starts_with?(pdf_binary, "%PDF")

      {:error, reason} ->
        flunk(
          "Expected successful PDF generation with nested iteration, got error: #{inspect(reason)}"
        )
    end
  end

  test "compile_to_pdf with list of maps iteration" do
    template = """
    = Employee Directory

    #for employee in json_data.employees [
      == #employee.name

      *Position:* #employee.position
      *Department:* #employee.department
      *Email:* #employee.email

      ---
    ]
    """

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
        },
        %{
          "name" => "Carol Davis",
          "position" => "Designer",
          "department" => "Design",
          "email" => "carol@company.com"
        }
      ]
    }

    case Imprintor.compile_to_pdf(template, data) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0
        assert String.starts_with?(pdf_binary, "%PDF")

      {:error, reason} ->
        flunk(
          "Expected successful PDF generation with map iteration, got error: #{inspect(reason)}"
        )
    end
  end

  test "compile_to_pdf with product catalog iteration" do
    template = """
    = Product Catalog

    #for product in json_data.products [
      *Price:* \\$ #product.price
      *Quantity:* #product.quantity
      ---
    ]

    = Summary
    """

    data = %{
      "products" => [
        %{
          "name" => "Laptop Pro",
          "price" => 1299,
          "quantity" => 15
        },
        %{
          "name" => "Wireless Mouse",
          "price" => 49,
          "quantity" => 50
        },
        %{
          "name" => "Gaming Keyboard",
          "price" => 129,
          "quantity" => 0
        }
      ]
    }

    case Imprintor.compile_to_pdf(template, data) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0
        assert String.starts_with?(pdf_binary, "%PDF")

      {:error, reason} ->
        flunk(
          "Expected successful PDF generation with product catalog, got error: #{inspect(reason)}"
        )
    end
  end
end
