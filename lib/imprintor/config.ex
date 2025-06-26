defmodule Imprintor.Config do
  defstruct [:source_document, :extra_fonts, :package_directory, :data]

  def new(source_document, data \\ %{}, opts \\ []) do
    extra_fonts = Keyword.get(opts, :extra_fonts, [])

    package_directory =
      Keyword.get(opts, :package_directory, System.get_env("TYPST_PACKAGE_DIRECTORY", ""))

    %__MODULE__{
      source_document: source_document,
      extra_fonts: extra_fonts,
      package_directory: package_directory,
      data: data
    }
  end
end
