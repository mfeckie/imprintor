defmodule Imprintor.Config do
  defstruct [:source_document, :extra_fonts, :root_directory, :data]

  def new(source_document, data \\ %{}, opts \\ []) do
    extra_fonts = Keyword.get(opts, :extra_fonts, [])

    root_directory = Keyword.get(opts, :root_directory, ".")

    %__MODULE__{
      source_document: source_document,
      extra_fonts: extra_fonts,
      root_directory: root_directory,
      data: data
    }
  end
end
