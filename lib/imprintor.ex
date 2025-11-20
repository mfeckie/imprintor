defmodule Imprintor do
  @moduledoc """
  Imprintor is a library for generating PDF documents from Typst templates.

  It provides functions to compile Typst templates with data interpolation
  and generate PDF documents using a native Rust implementation.
  """

  mix_config = Mix.Project.config()
  version = mix_config[:version]

  use RustlerPrecompiled,
    otp_app: :imprintor,
    crate: :imprintor,
    version: version,
    base_url: "https://github.com/mfeckie/imprintor/releases/download/#{version}",
    force_build: System.get_env("FORCE_COMPILE") in ["true", "1"],
    nif_versions: [
      "2.15",
      "2.16",
      "2.17"
    ],
    targets: [
      "aarch64-apple-darwin",
      "x86_64-unknown-linux-gnu",
      "x86_64-unknown-linux-musl",
      "aarch64-unknown-linux-gnu",
      "aarch64-unknown-linux-musl"
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

  def typst_to_pdf(_config), do: :erlang.nif_error(:nif_not_loaded)
  def typst_to_pdf_file(_config, _output_path), do: :erlang.nif_error(:nif_not_loaded)
end
