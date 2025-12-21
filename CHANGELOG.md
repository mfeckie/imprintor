## Changelog

### 0.1.0

- Initial release
- Basic Typst template compilation using native Rust
- PDF generation with data access using `elixir_data` variable  
- Support for nested data structures and array iteration
- Configuration-based API with `Imprintor.Config`
- Custom fonts support
- Comprehensive test suite

## 0.3.0

Breaking Change - All externally supplied data is now to be accessed via `sys.inputs.elixir_data`, this is to make things more in keeping with how the CLI for Typst works.

If you were previously accessing `elixir_data` global, just change to `sys.inputs.elixir_data`


## 0.4.0

- **Added:** `Imprintor.compile_to_pdf_file/2` — convenience function to compile a Typst template and write the resulting PDF directly to disk (returns `{:ok, path}` or `{:error, reason}`).

## 0.5.0

- Adds windows as a precompiled target [via](https://github.com/mfeckie/imprintor/pull/11)
- Bumps typst to 0.14.2
- Updates dependencies