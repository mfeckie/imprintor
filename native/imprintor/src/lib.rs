use rustler::types::binary;
use rustler::{NifStruct, Term};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use typst::diag::{FileError, FileResult, PackageError, PackageResult};
use typst::ecow::eco_format;
use typst::foundations::{Array, Dict, Str, Value};
use typst::foundations::{Bytes, Datetime, Duration};
use typst::syntax::package::PackageSpec;
use typst::syntax::{FileId, Source, VirtualRoot};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};
use typst_kit::fonts::{self, FontStore};
use typst_layout::PagedDocument;
use typst_pdf::{PdfOptions, PdfStandard, PdfStandards};
use typst_render::RenderOptions;
/// A File that will be stored in the HashMap.
#[derive(Clone, Debug)]
struct FileEntry {
    bytes: Bytes,
    source: Option<Source>,
}

rustler::atoms! {
    ok,
    error,
}

impl FileEntry {
    fn new(bytes: Vec<u8>, source: Option<Source>) -> Self {
        Self {
            bytes: Bytes::new(bytes),
            source,
        }
    }

    fn source(&mut self, id: FileId) -> FileResult<Source> {
        let source = if let Some(source) = &self.source {
            source
        } else {
            let contents = std::str::from_utf8(&self.bytes).map_err(|_| FileError::InvalidUtf8)?;
            let contents = contents.trim_start_matches('\u{feff}');
            let source = Source::new(id, contents.into());
            self.source.insert(source)
        };
        Ok(source.clone())
    }
}

struct ImprintorNifWorld {
    root: PathBuf,
    source: Source,
    library: LazyHash<Library>,
    fonts: FontStore,
    files: Arc<Mutex<HashMap<FileId, FileEntry>>>,
    time: time::OffsetDateTime,
    cache_directory: PathBuf,
}
#[derive(NifStruct)]
#[module = "Imprintor.Config"]
pub struct ImprintorConfig<'a> {
    source_document: String,
    extra_fonts: Option<Vec<String>>,
    data: Option<Term<'a>>,
    root_directory: String,
    pdf_standard: Option<String>,
    ppi: Option<f64>,
}

impl ImprintorNifWorld {
    fn new(config: ImprintorConfig) -> Self {
        let root = PathBuf::from(config.root_directory);

        let mut font_store = FontStore::new();
        font_store.extend(fonts::system());
        font_store.extend(fonts::embedded());
        if let Some(extra_fonts) = config.extra_fonts {
            for path in &extra_fonts {
                font_store.extend(fonts::scan(std::path::Path::new(path)));
            }
        }

        let mut dict = Dict::new();

        if let Some(elixir_data) = config.data {
            let typst_value = typst_values_from_elxiir(elixir_data);

            dict.insert("elixir_data".into(), typst_value);
        }

        let library = Library::builder().with_inputs(dict).build();

        let cache_directory = std::env::var_os("CACHE_DIRECTORY")
            .map(|os_path| os_path.into())
            .unwrap_or(std::env::temp_dir());

        Self {
            source: Source::detached(config.source_document),
            fonts: font_store,
            time: time::OffsetDateTime::now_utc(),
            library: LazyHash::new(library),
            files: Arc::new(Mutex::new(HashMap::new())),
            root,
            cache_directory,
        }
    }

    /// Helper to handle file requests.
    ///
    /// Requests will be either in packages or a local file.
    fn file(&self, id: FileId) -> FileResult<FileEntry> {
        let mut files = self.files.lock().map_err(|_| FileError::AccessDenied)?;
        if let Some(entry) = files.get(&id) {
            return Ok(entry.clone());
        }
        let path = match id.root() {
            VirtualRoot::Package(package) => {
                // Fetching file from package
                let package_dir = self.download_package(package)?;
                id.vpath().realize(&package_dir)
            }
            VirtualRoot::Project => {
                // Fetching file from disk
                id.vpath().realize(&self.root)
            }
        }
        .map_err(|_| FileError::AccessDenied)?;

        let content = std::fs::read(&path).map_err(|error| FileError::from_io(error, &path))?;
        Ok(files
            .entry(id)
            .or_insert(FileEntry::new(content, None))
            .clone())
    }

    /// Downloads the package and returns the system path of the unpacked package.
    fn download_package(&self, package: &PackageSpec) -> PackageResult<PathBuf> {
        let package_subdir = format!("{}/{}/{}", package.namespace, package.name, package.version);
        let path = self.cache_directory.join(package_subdir);

        if path.exists() {
            return Ok(path);
        }

        eprintln!("downloading {package}");
        let url = format!(
            "https://packages.typst.org/{}/{}-{}.tar.gz",
            package.namespace, package.name, package.version,
        );

        let mut response = retry(|| {
            let response = ureq::get(&url)
                .call()
                .map_err(|error| eco_format!("{error}"))?;

            let status = response.status();
            if !http_successful(status.into()) {
                return Err(eco_format!(
                    "response returned unsuccessful status code {status}",
                ));
            }

            Ok(response)
        })
        .map_err(|error| PackageError::NetworkFailed(Some(error)))?;

        let compressed_archive = response
            .body_mut()
            .read_to_vec()
            .map_err(|error| PackageError::NetworkFailed(Some(eco_format!("{error}"))))?;

        let raw_archive = zune_inflate::DeflateDecoder::new(&compressed_archive)
            .decode_gzip()
            .map_err(|error| PackageError::MalformedArchive(Some(eco_format!("{error}"))))?;

        let mut archive = tar::Archive::new(raw_archive.as_slice());

        archive.unpack(&path).map_err(|error| {
            _ = std::fs::remove_dir_all(&path);
            PackageError::MalformedArchive(Some(eco_format!("{error}")))
        })?;

        Ok(path)
    }
}

fn retry<T, E>(mut f: impl FnMut() -> Result<T, E>) -> Result<T, E> {
    if let Ok(ok) = f() {
        Ok(ok)
    } else {
        f()
    }
}

fn http_successful(status: u16) -> bool {
    // 2XX
    status / 100 == 2
}

fn typst_values_from_elxiir(term: Term) -> typst::foundations::Value {
    match term.get_type() {
        rustler::TermType::Atom => {
            let atom = term.atom_to_string().unwrap();
            Value::Str(atom.into())
        }
        rustler::TermType::Binary => {
            let binary: binary::Binary = term.decode().unwrap();
            let string = String::from_utf8(binary.to_vec()).unwrap();
            Value::Str(string.into())
        }
        rustler::TermType::List => {
            let list: Vec<Term> = term.decode().unwrap();
            let typst_array: Array = list.into_iter().map(typst_values_from_elxiir).collect();
            Value::Array(typst_array)
        }
        rustler::TermType::Tuple => {
            let tuple: (Term, Term) = term.decode().unwrap();

            if tuple.0.get_type() == rustler::TermType::Atom
                && tuple.0.atom_to_string().is_ok_and(|atom| atom == "bytes")
            {
                let binary: binary::Binary = tuple.1.decode().unwrap();
                Value::Bytes(Bytes::new(binary.to_vec()))
            } else {
                Value::None
            }
        }
        rustler::TermType::Map => {
            let map: HashMap<Term, Term> = term.decode().unwrap();
            let mut dict = Dict::new();

            for (key, value) in map {
                let key_str = match key.get_type() {
                    rustler::TermType::Atom => key.atom_to_string().unwrap(),
                    rustler::TermType::Binary => {
                        let binary: binary::Binary = key.decode().unwrap();
                        String::from_utf8(binary.to_vec()).unwrap()
                    }
                    _ => continue, // Skip unsupported key types
                };
                dict.insert(Str::from(key_str), typst_values_from_elxiir(value));
            }
            Value::Dict(dict)
        }
        rustler::TermType::Integer => {
            let int: i64 = term.decode().unwrap();
            Value::Int(int)
        }
        rustler::TermType::Float => {
            let float: f64 = term.decode().unwrap();
            Value::Float(float)
        }
        _ => Value::None, // Handle other types as needed
    }
}

/// This is the interface we have to implement such that `typst` can compile it.
///
/// I have tried to keep it as minimal as possible
impl typst::World for ImprintorNifWorld {
    /// Standard library.
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Metadata about all known Books.
    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    /// Accessing the main source file.
    fn main(&self) -> FileId {
        self.source.id()
    }

    /// Accessing a specified source file (based on `FileId`).
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            self.file(id)?.source(id)
        }
    }

    /// Accessing a specified file (non-file).
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.file(id).map(|file| file.bytes.clone())
    }

    /// Accessing a specified font per index of font book.
    fn font(&self, id: usize) -> Option<Font> {
        self.fonts.font(id)
    }

    /// Get the current date.
    ///
    /// Optionally, an offset in hours is given.
    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        let offset = offset.map(|offset| offset.hours() as i64).unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

/// Clears Typst's global memoization cache when dropped.
///
/// Typst memoizes compilation and export work in a process-wide cache
/// (`comemo`) that only shrinks when the host calls `comemo::evict`. Without
/// eviction, entries accumulate for the life of the BEAM (native memory that
/// the VM can't see or reclaim). Keeping them between calls made no measurable
/// difference to compile time, so the cache is cleared after every call.
/// Declaring this guard first in each
/// NIF makes it drop last, after the world and document, and also on early
/// returns and panics.
struct EvictCacheOnDrop;

impl Drop for EvictCacheOnDrop {
    fn drop(&mut self) {
        typst::comemo::evict(0);
    }
}

/// Compiles the world's source into a paged document, formatting any
/// diagnostics into a single error string on failure.
fn compile_document(world: &ImprintorNifWorld) -> Result<PagedDocument, String> {
    match typst::compile::<PagedDocument>(world).output {
        Ok(document) => Ok(document),
        Err(errors) => {
            let error_msg = errors
                .iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ");
            Err(format!("Compilation failed: {}", error_msg))
        }
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn typst_to_pdf<'a>(
    env: rustler::Env<'a>,
    config: ImprintorConfig,
) -> Result<rustler::Binary<'a>, String> {
    let _evict_cache = EvictCacheOnDrop;
    let pdf_options = build_pdf_options(config.pdf_standard.as_deref())?;
    let world = ImprintorNifWorld::new(config);
    let document = compile_document(&world)?;

    let pdf_bytes = typst_pdf::pdf(&document, &pdf_options).unwrap();
    let mut binary = rustler::OwnedBinary::new(pdf_bytes.len()).unwrap();
    binary.as_mut_slice().copy_from_slice(&pdf_bytes);
    Ok(binary.release(env))
}

#[rustler::nif(schedule = "DirtyCpu")]
fn typst_to_pdf_file<'a>(config: ImprintorConfig, output_path: String) -> Result<String, String> {
    let _evict_cache = EvictCacheOnDrop;
    let pdf_options = build_pdf_options(config.pdf_standard.as_deref())?;
    let world = ImprintorNifWorld::new(config);
    let document = compile_document(&world)?;

    let pdf_bytes = typst_pdf::pdf(&document, &pdf_options).unwrap();
    std::fs::write(&output_path, pdf_bytes).map_err(|err| err.to_string())?;
    Ok(output_path)
}

#[rustler::nif(schedule = "DirtyCpu")]
fn typst_to_png<'a>(
    env: rustler::Env<'a>,
    config: ImprintorConfig,
) -> Result<Vec<rustler::Binary<'a>>, String> {
    let _evict_cache = EvictCacheOnDrop;
    let ppi = config.ppi;
    let world = ImprintorNifWorld::new(config);
    let document = compile_document(&world)?;
    let pages = render_document_to_pngs(&document, ppi)?;

    pages
        .into_iter()
        .map(|png_bytes| {
            let mut binary = rustler::OwnedBinary::new(png_bytes.len()).unwrap();
            binary.as_mut_slice().copy_from_slice(&png_bytes);
            Ok(binary.release(env))
        })
        .collect()
}

#[rustler::nif(schedule = "DirtyCpu")]
fn typst_to_png_file(config: ImprintorConfig, output_path: String) -> Result<Vec<String>, String> {
    let _evict_cache = EvictCacheOnDrop;
    let ppi = config.ppi;
    let world = ImprintorNifWorld::new(config);
    let document = compile_document(&world)?;
    let pages = render_document_to_pngs(&document, ppi)?;

    let output_paths = page_output_paths(&output_path, pages.len());

    for (index, (path, png_bytes)) in output_paths.iter().zip(pages).enumerate() {
        if let Err(err) = std::fs::write(path, png_bytes) {
            for written_path in &output_paths[..index] {
                let _ = std::fs::remove_file(written_path);
            }
            return Err(err.to_string());
        }
    }

    Ok(output_paths)
}

/// Renders every page of a document to PNG bytes at the given pixel density.
///
/// `ppi` defaults to 144 (matching `typst-cli`'s default) when not given.
fn render_document_to_pngs(
    document: &PagedDocument,
    ppi: Option<f64>,
) -> Result<Vec<Vec<u8>>, String> {
    let options = build_render_options(ppi);

    document
        .pages()
        .iter()
        .map(|page| {
            typst_render::render(page, &options)
                .encode_png()
                .map_err(|err| format!("Failed to encode PNG: {err}"))
        })
        .collect()
}

fn build_render_options(ppi: Option<f64>) -> RenderOptions {
    let ppi = ppi.unwrap_or(144.0);

    RenderOptions {
        pixel_per_pt: typst::utils::Scalar::new(ppi / 72.0),
        render_bleed: false,
    }
}

/// Derives one output path per page. A single-page document is written
/// directly to `output_path`; multi-page documents get the 1-based page
/// number inserted before the file extension (e.g. `out.png` -> `out-1.png`,
/// `out-2.png`, ...).
fn page_output_paths(output_path: &str, page_count: usize) -> Vec<String> {
    if page_count <= 1 {
        return vec![output_path.to_string()];
    }

    let path = std::path::Path::new(output_path);
    let extension = path.extension().and_then(|ext| ext.to_str());
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or(output_path);
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());

    (1..=page_count)
        .map(|page_number| {
            let file_name = match extension {
                Some(extension) => format!("{stem}-{page_number}.{extension}"),
                None => format!("{stem}-{page_number}"),
            };

            match parent {
                Some(parent) => parent.join(file_name).to_string_lossy().into_owned(),
                None => file_name,
            }
        })
        .collect()
}

fn build_pdf_options(pdf_standard: Option<&str>) -> Result<PdfOptions, String> {
    let mut options = PdfOptions::default();

    if let Some(standard_value) = pdf_standard {
        let standard = parse_pdf_standard(standard_value).ok_or_else(|| {
            format!(
                "Unsupported PDF standard '{}' . Supported values: {}",
                standard_value,
                SUPPORTED_PDF_STANDARDS.join(", ")
            )
        })?;

        options.standards = PdfStandards::new(&[standard])
            .map_err(|err| format!("Invalid PDF standard: {err:?}"))?;
    }

    Ok(options)
}

const SUPPORTED_PDF_STANDARDS: [&str; 17] = [
    "1.4", "1.5", "1.6", "1.7", "2.0", "a-1a", "a-1b", "a-2a", "a-2b", "a-2u", "a-3a", "a-3b",
    "a-3u", "a-4", "a-4e", "a-4f", "ua-1",
];

fn parse_pdf_standard(input: &str) -> Option<PdfStandard> {
    match input.trim().to_ascii_lowercase().as_str() {
        "1.4" => Some(PdfStandard::V_1_4),
        "1.5" => Some(PdfStandard::V_1_5),
        "1.6" => Some(PdfStandard::V_1_6),
        "1.7" => Some(PdfStandard::V_1_7),
        "2.0" => Some(PdfStandard::V_2_0),
        "a-1a" => Some(PdfStandard::A_1a),
        "a-1b" => Some(PdfStandard::A_1b),
        "a-2a" => Some(PdfStandard::A_2a),
        "a-2b" => Some(PdfStandard::A_2b),
        "a-2u" => Some(PdfStandard::A_2u),
        "a-3a" => Some(PdfStandard::A_3a),
        "a-3b" => Some(PdfStandard::A_3b),
        "a-3u" => Some(PdfStandard::A_3u),
        "a-4" => Some(PdfStandard::A_4),
        "a-4e" => Some(PdfStandard::A_4e),
        "a-4f" => Some(PdfStandard::A_4f),
        "ua-1" => Some(PdfStandard::Ua_1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `build_pdf_options` only ever calls `PdfStandards::new` with a single
    /// standard, so it can never hit the conflicting-standards error branch.
    /// This exercises that branch directly against the underlying crate to
    /// confirm `PdfStandards::new`'s error type is still `Debug`-formattable
    /// (the `{err:?}` in `build_pdf_options` relies on this after typst-pdf
    /// 0.15 dropped `Display` from `HintedString`).
    #[test]
    fn conflicting_pdf_standards_error_is_debug_formattable() {
        let result = PdfStandards::new(&[PdfStandard::V_1_4, PdfStandard::V_1_7]);
        let err = result.expect_err("conflicting PDF version standards should fail to construct");
        let message = format!("Invalid PDF standard: {err:?}");
        assert!(!message.is_empty());
    }
}

rustler::init!("Elixir.Imprintor");
