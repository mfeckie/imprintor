defmodule Imprintor do
  @moduledoc """
  Imprintor is a library for generating PDF documents from Typst templates.

  It provides functions to compile Typst templates with data interpolation
  and generate PDF documents using a native Rust implementation.
  """

  use Rustler, otp_app: :imprintor, crate: "imprintor"

  @doc """
  """
  def compile_to_pdf(config) do
    case typst_to_pdf(config) do
      {:ok, pdf_binary} -> {:ok, pdf_binary}
      {:error, reason} -> {:error, reason}
      pdf_binary when is_binary(pdf_binary) -> {:ok, pdf_binary}
      error -> {:error, error}
    end
  end

  # Private NIF function - called by compile_to_pdf/2

  def typst_to_pdf(_config), do: :erlang.nif_error(:nif_not_loaded)
end
