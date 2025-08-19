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
      "aarch64-unknown-linux-gnu"
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

  def typst_to_pdf(_config), do: :erlang.nif_error(:nif_not_loaded)
end
