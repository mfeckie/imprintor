defmodule ImprintorTest do
  use ExUnit.Case
  doctest Imprintor

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

      *Total Sales:* \\#region.total_sales

      === Products
      #for product in region.products [
        - #product.name: \\#product.revenue
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
      *Price:* \\#product.price
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

  test "compile_to_pdf with typst package (QR code)" do
    template = """
    #import "@preview/cades:0.3.0": qr-code

    = QR Code Test

    This is a test of the qr-code plugin.

    #qr-code("https://typst.app")

    == Additional Info

    *Website:* #elixir_data.website
    *Generated on:* #elixir_data.date
    """

    data = %{
      "website" => "https://typst.app",
      "date" => "2024-01-15"
    }

    config = Imprintor.Config.new(template, data)

    case Imprintor.compile_to_pdf(config) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0
        assert String.starts_with?(pdf_binary, "%PDF")

      {:error, reason} ->
        flunk(
          "Expected successful PDF generation with Typst package, got error: #{inspect(reason)}"
        )
    end
  end

  test "compile_to_pdf with atom keys in data structure" do
    template = """
    = {{title}}

    *Author:* {{author}}
    *Version:* {{version}}

    == Features

    #for feature in elixir_data.features [
      - #feature.name: #feature.description
    ]

    == Status
    *Ready:* {{ready}}
    """

    data = %{
      title: "Project Documentation",
      author: "Development Team",
      version: "1.0.0",
      ready: true,
      features: [
        %{name: "Authentication", description: "User login and registration"},
        %{name: "Dashboard", description: "Main application interface"},
        %{name: "Reports", description: "Data visualization and export"}
      ]
    }

    config = Imprintor.Config.new(template, data)

    case Imprintor.compile_to_pdf(config) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0
        assert String.starts_with?(pdf_binary, "%PDF")

      {:error, reason} ->
        flunk("Expected successful PDF generation with atom keys, got error: #{inspect(reason)}")
    end
  end

  test "compile_to_pdf with large file - 500 items with images and QR codes" do
    template = """
    #let qr = plugin("test/typst_plugin_qr.wasm")
    #import "@preview/tiaoma:0.3.0"

    = Large Inventory Report

    *Generated:* #elixir_data.generated_date
    *Total Items:* #elixir_data.total_items

    == Product Inventory

    #for item in elixir_data.items [
      === Product \\#item.id: #item.name

      #grid(
        columns: (1fr, 1fr),
        gutter: 1em,
        [
          *SKU:* #item.sku \
          *Category:* #item.category \
          *Price:* \\$#item.price \
          *Stock:* #item.stock_quantity \
          *Status:* #item.status
        ],
        [
          *QR Code:* \
          #tiaoma.barcode("asdfasdf", "QRCode")
        ]
      )

      *Description:* #item.description

      *Product Image:*
      #rect(
        width: 4cm,
        height: 3cm,
        fill: gradient.linear(rgb("#e1f5fe"), rgb("#bbdefb")),
        stroke: 1pt + gray,
        [
          #align(center + horizon)[
            #text(size: 0.8em, fill: gray)[
              Image: #item.image_url
            ]
          ]
        ]
      )

      *Barcode:* #item.barcode

      #line(length: 100%, stroke: 0.5pt + gray)
      #v(0.5em)
    ]

    == Summary Statistics

    *Total Value:* \\$#elixir_data.total_value
    *Categories:* #elixir_data.category_count
    *Low Stock Items:* #elixir_data.low_stock_count
    """

    # Generate data for 500 items
    items =
      Enum.map(1..500, fn i ->
        %{
          id: i,
          name: "Product #{i}",
          sku: "SKU-#{String.pad_leading(Integer.to_string(i), 6, "0")}",
          category: Enum.random(["Electronics", "Clothing", "Home & Garden", "Sports", "Books"]),
          price: :rand.uniform(1000) + 10,
          stock_quantity: :rand.uniform(100),
          status: if(:rand.uniform() > 0.8, do: "Low Stock", else: "In Stock"),
          description: "High-quality product #{i} with excellent features and durability.",
          qr_data: "https://inventory.example.com/product/#{i}",
          image_url: "product_#{i}.jpg",
          barcode: "#{:rand.uniform(999_999_999_999)}"
        }
      end)

    total_value = Enum.reduce(items, 0, fn item, acc -> acc + item.price end)
    categories = items |> Enum.map(& &1.category) |> Enum.uniq() |> length()
    low_stock_count = Enum.count(items, fn item -> item.status == "Low Stock" end)

    data = %{
      generated_date: "2025-07-01",
      total_items: 500,
      items: items,
      total_value: total_value,
      category_count: categories,
      low_stock_count: low_stock_count
    }

    config = Imprintor.Config.new(template, data)

    case Imprintor.compile_to_pdf(config) do
      {:ok, pdf_binary} ->
        assert is_binary(pdf_binary)
        assert byte_size(pdf_binary) > 0
        assert String.starts_with?(pdf_binary, "%PDF")
        # Verify it's a substantial file due to 500 items
        # Should be at least 100KB
        assert byte_size(pdf_binary) > 100_000

      {:error, reason} ->
        flunk(
          "Expected successful PDF generation with large file (500 items), got error: #{inspect(reason)}"
        )
    end
  end
end
