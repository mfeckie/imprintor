defmodule Imprintor do
  @moduledoc """
  Imprintor is a library for generating PDF and PNG documents from Typst templates.

  It provides functions to compile Typst templates with data interpolation
  and generate PDF or PNG output using a native Rust implementation.
  """

  mix_config = Mix.Project.config()
  version = mix_config[:version]

  use RustlerPrecompiled,
    otp_app: :imprintor,
    crate: :imprintor,
    version: version,
    base_url: "https://github.com/mfeckie/imprintor/releases/download/#{version}",
    # Passing `force_build:` overrides RustlerPrecompiled's own
    # `config :rustler_precompiled, :force_build, imprintor: true`, so honour
    # that config here as well as the FORCE_COMPILE environment variable.
    force_build:
      System.get_env("FORCE_COMPILE") in ["true", "1"] or
        Application.compile_env(:rustler_precompiled, [:force_build, :imprintor], false),
    nif_versions: [
      "2.15",
      "2.16",
      "2.17"
    ],
    targets: [
      "aarch64-apple-darwin",
      "x86_64-apple-darwin",
      "x86_64-unknown-linux-gnu",
      "x86_64-unknown-linux-musl",
      "aarch64-unknown-linux-gnu",
      "aarch64-unknown-linux-musl",
      "arm-unknown-linux-gnueabihf",
      "x86_64-pc-windows-msvc"
    ]

  @doc """
  Compiles a Typst template to a PDF document.

  Takes an `Imprintor.Config` struct containing the template configuration and
  returns a binary containing the compiled PDF data.

  ## Parameters

    * `config` - An `%Imprintor.Config{}` struct containing:
      * Template source or file path
      * Data for interpolation
      * Compilation options

  ## Returns

    * `{:ok, pdf_binary}` - Successfully compiled PDF as binary data
    * `{:error, reason}` - Compilation failed with error reason
  """
  def compile_to_pdf(%Imprintor.Config{} = config) do
    case typst_to_pdf(config) do
      {:ok, pdf_binary} -> {:ok, pdf_binary}
      {:error, reason} -> {:error, reason}
      pdf_binary when is_binary(pdf_binary) -> {:ok, pdf_binary}
      error -> {:error, error}
    end
  end

  @doc """
  Compiles a Typst template to a PDF file.

  Takes an `Imprintor.Config` struct and an output file path, compiles the
  template, and writes the resulting PDF to the specified file. 

  ## Parameters

    * `config` - An `%Imprintor.Config{}` struct containing:
      * Template source or file path
      * Data for interpolation
      * Compilation options
    * `output_path` - A string specifying the file path to write the PDF to
  """

  def compile_to_pdf_file(%Imprintor.Config{} = config, output_path)
      when is_binary(output_path) do
    case typst_to_pdf_file(config, output_path) do
      {:ok, _path} = result -> result
      {:error, reason} -> {:error, reason}
      error -> {:error, error}
    end
  end

  @doc """
  Compiles a Typst template to one or more PNG images, one per page.

  Takes an `Imprintor.Config` struct containing the template configuration and
  returns a list of binaries, each containing the compiled PNG data for one
  page, in page order.

  ## Parameters

    * `config` - An `%Imprintor.Config{}` struct containing:
      * Template source or file path
      * Data for interpolation
      * Compilation options
      * `:ppi` - Pixels-per-inch used for rendering (defaults to `144.0`)

  ## Returns

    * `{:ok, [png_binary, ...]}` - Successfully compiled PNG(s) as binary data
    * `{:error, reason}` - Compilation failed with error reason
  """
  def compile_to_png(%Imprintor.Config{} = config) do
    case typst_to_png(config) do
      {:ok, png_binaries} -> {:ok, png_binaries}
      {:error, reason} -> {:error, reason}
      png_binaries when is_list(png_binaries) -> {:ok, png_binaries}
      error -> {:error, error}
    end
  end

  @doc """
  Compiles a Typst template to one or more PNG files, one per page.

  Takes an `Imprintor.Config` struct and an output file path, compiles the
  template, and writes the resulting PNG(s) to disk.

  For a single-page document, the image is written directly to
  `output_path`. For a multi-page document, the 1-based page number is
  inserted before the file extension for each page (e.g. `report.png` becomes
  `report-1.png`, `report-2.png`, ...).

  ## Parameters

    * `config` - An `%Imprintor.Config{}` struct containing:
      * Template source or file path
      * Data for interpolation
      * Compilation options
      * `:ppi` - Pixels-per-inch used for rendering (defaults to `144.0`)
    * `output_path` - A string specifying the file path to write the PNG(s) to

  ## Returns

    * `{:ok, [path, ...]}` - Successfully wrote the PNG(s), in page order
    * `{:error, reason}` - Compilation or writing failed with error reason
  """
  def compile_to_png_file(%Imprintor.Config{} = config, output_path)
      when is_binary(output_path) do
    case typst_to_png_file(config, output_path) do
      {:ok, _paths} = result -> result
      {:error, reason} -> {:error, reason}
      error -> {:error, error}
    end
  end

  @doc """
  Wraps raw binary data to be consumed as Typst `bytes`.

  This is useful for APIs like `pdf.attach` that expect a bytes value.

  ## Examples

      data = %{
        "factur_x_xml" => Imprintor.bytes("<xml>...</xml>")
      }
  """
  def bytes(binary) when is_binary(binary), do: {:bytes, binary}

  def typst_to_pdf(_config), do: :erlang.nif_error(:nif_not_loaded)
  def typst_to_pdf_file(_config, _output_path), do: :erlang.nif_error(:nif_not_loaded)
  def typst_to_png(_config), do: :erlang.nif_error(:nif_not_loaded)
  def typst_to_png_file(_config, _output_path), do: :erlang.nif_error(:nif_not_loaded)
end
