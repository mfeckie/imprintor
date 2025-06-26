defmodule ImprintorTest do
  use ExUnit.Case

  test "compile_to_pdf with simple template" do
    template = "= Hello {{name}}\n\nThis is a test document."
    data = %{"name" => "World"}

    config = Imprintor.Config.new(template, data)

    case Imprintor.compile_to_pdf(config) do
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

    config = Imprintor.Config.new(template)

    case Imprintor.compile_to_pdf(config) do
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

    config = Imprintor.Config.new(template, data)

    case Imprintor.compile_to_pdf(config) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0

      {:error, reason} ->
        flunk("Expected successful PDF generation, got error: #{inspect(reason)}")
    end
  end

  test "compile_to_pdf with simple list iteration" do
    template = """
    = Simple List Test

    #for item in elixir_data.items [
      - #item
    ]
    """

    data = %{
      "items" => ["Apple", "Banana", "Cherry"]
    }

    config = Imprintor.Config.new(template, data)

    case Imprintor.compile_to_pdf(config) do
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

    case Imprintor.compile_to_pdf(config) do
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

    #for employee in elixir_data.employees [
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

    config = Imprintor.Config.new(template, data)

    case Imprintor.compile_to_pdf(config) do
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

    #for product in elixir_data.products [
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

    config = Imprintor.Config.new(template, data)

    case Imprintor.compile_to_pdf(config) do
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
