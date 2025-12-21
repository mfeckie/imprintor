defmodule Imprintor.MixProject do
  use Mix.Project

  @version "0.5.0"
  @source_url "https://github.com/mfeckie/imprintor"

  def project do
    [
      app: :imprintor,
      description: "Imprintor is a library for generating PDF documents from Typst templates.",
      deps: deps(),
      elixir: "~> 1.17",
      package: package(),
      start_permanent: Mix.env() == :prod,
      version: @version,
      docs: [
        source_url: @source_url,
        source_ref: @version,
        extras: ["README.md"],
        main: "readme"
      ]
    ]
  end

  # Run "mix help compile.app" to learn about applications.
  def application do
    [
      extra_applications: [:logger]
    ]
  end

  defp package do
    [
      licenses: ["MIT"],
      links: %{"GitHub" => @source_url},
      files: [
        "lib",
        "native",
        "checksum-*.exs",
        "mix.exs"
      ],
      exclude_patterns: [
        "native/imprintor/target"
      ]
    ]
  end

  # Run "mix help deps" to learn about dependencies.
  defp deps do
    [
      {:rustler, ">= 0.0.0", optional: true},
      {:rustler_precompiled, "~> 0.8"},
      {:jason, "~> 1.4"},
      {:ex_doc, "~> 0.34", only: :dev, runtime: false, warn_if_outdated: true}
    ]
  end
end
