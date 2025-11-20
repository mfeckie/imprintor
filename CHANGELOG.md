## 0.3.0

Breaking Change - All externally supplied data is now to be accessed via `sys.inputs.elixir_data`, this is to make things more in keeping with how the CLI for Typst works.

If you were previously accessing `elixir_data` global, just change to `sys.inputs.elixir_data`