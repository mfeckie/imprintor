## Changelog

## Unreleased

- **Fixed:** native memory no longer grows with every compile. Typst keeps a process-wide memoization cache that is only freed when the host evicts it, and Imprintor never did, so every `compile_to_pdf`/`compile_to_png` call (and the `_file` variants) left its cache entries behind for the life of the BEAM. This memory is outside the VM, so it doesn't show up in `:erlang.memory/0`, only in the OS process size. Each call now clears the cache when it finishes. Calls never shared cache entries, because each one builds a fresh world, so this doesn't make compiles slower.

## 0.9.0

- **Changed:** precompiled NIFs are no longer built for NIF version 2.15 (OTP 22/23). This has no effect on users because imprintor requires Elixir 1.17+ (OTP 25+), which uses NIF 2.16+.
- **Fixed:** `rustler_precompiled` is updated to 0.10.1 (now required, `~> 0.10`), which fixes NIF downloads from GitHub releases that failed with `{:header_too_long, {:max, 10240}}`. Rust dependency lockfile updates are included.
- **Added:** precompiled NIFs are now built for NIF version 2.18 (Erlang/OTP 29).
- **Changed:** the `x86_64-apple-darwin` NIF is now cross-compiled on an Apple Silicon (`macos-15`) runner instead of `macos-15-intel`, because GitHub is retiring the Intel macOS runner images.

## 0.8.0

- **Added:** precompiled NIFs for Intel Macs (`x86_64-apple-darwin`, built on the `macos-15-intel` runner). `arm-unknown-linux-gnueabihf` is now in `targets:`; it was already built and published, but could never be downloaded.
- **Fixed:** `config :rustler_precompiled, :force_build, imprintor: true` is now honoured. Imprintor always passed `force_build:` to RustlerPrecompiled, which overrode that config, so only `FORCE_COMPILE=1` could force a source build.
- **Docs:** the README lists the precompiled targets and explains how to build from source: Rust 1.92+, adding `:rustler`, and using `mix compile` rather than `mix deps.compile imprintor`. Its examples now use `sys.inputs.elixir_data` (a bare `elixir_data` fails with `unknown variable: elixir_data`).

## 0.7.0

- **Added:** `Imprintor.compile_to_png/1` and `Imprintor.compile_to_png_file/2` — render a Typst template to PNG image(s) instead of a PDF ([#16](https://github.com/mfeckie/imprintor/issues/16)). Multi-page documents produce one binary/file per page; `Imprintor.Config` gains an optional `:ppi` field (defaults to `144.0`, matching `typst-cli`).
- Bumps typst/typst-kit/typst-pdf to 0.15.1 and rustler to 0.38.0.
  - **Note:** generated PDFs are now minified by default (Typst 0.15 changed this default), so output bytes will differ from previous versions even though content is unchanged.
  - **Note:** generated PDFs are now tagged for accessibility by default (Typst 0.15 always writes a baseline-tagged PDF, even without requesting the `ua-1` standard).
  - **Note:** if you reference a variable font by a name like `"Inter Variable"` via `:extra_fonts`, Typst 0.15 now trims `Variable`/`Var`/`VF` suffixes from font family names — use the base family name (e.g. `"Inter"`) instead.
  - **Note:** if your Typst templates use backslashes in `#import`/`#include`/`image()` paths (e.g. on Windows), Typst 0.15 no longer allows backslashes in these paths — use forward slashes.

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