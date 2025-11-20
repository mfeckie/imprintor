use rustler::types::binary;
use rustler::{NifStruct, Term};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use typst::diag::{FileError, FileResult, PackageError, PackageResult};
use typst::ecow::eco_format;
use typst::foundations::{Array, Dict, Str, Value};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::package::PackageSpec;
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, LibraryExt};
use typst_kit::fonts::{FontSlot, Fonts};
use typst_pdf::PdfOptions;
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

#[derive(Debug)]
struct ImprintorNifWorld {
    root: PathBuf,
    source: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
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
}

impl ImprintorNifWorld {
    fn new(config: ImprintorConfig) -> Self {
        let root = PathBuf::from(config.root_directory);

        let font_searcher = match config.extra_fonts {
            Some(fonts) => Fonts::searcher()
                .include_system_fonts(true)
                .search_with(fonts),
            None => Fonts::searcher().include_system_fonts(true).search(),
        };

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
            fonts: font_searcher.fonts,
            time: time::OffsetDateTime::now_utc(),
            book: LazyHash::new(font_searcher.book),
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
        let path = if let Some(package) = id.package() {
            // Fetching file from package
            let package_dir = self.download_package(package)?;
            id.vpath().resolve(&package_dir)
        } else {
            // Fetching file from disk
            id.vpath().resolve(&self.root)
        }
        .ok_or(FileError::AccessDenied)?;

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
        &self.book
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
        self.fonts[id].get()
    }

    /// Get the current date.
    ///
    /// Optionally, an offset in hours is given.
    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

#[rustler::nif(schedule = "DirtyCpu")]
fn typst_to_pdf<'a>(
    env: rustler::Env<'a>,
    config: ImprintorConfig,
) -> Result<rustler::Binary<'a>, String> {
    let world = ImprintorNifWorld::new(config);

    match typst::compile(&world).output {
        Ok(document) => {
            let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default()).unwrap();
            let mut binary = rustler::OwnedBinary::new(pdf_bytes.len()).unwrap();
            binary.as_mut_slice().copy_from_slice(&pdf_bytes);
            Ok(binary.release(env))
        }
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
fn typst_to_pdf_file<'a>(config: ImprintorConfig, output_path: String) -> Result<String, String> {
    let world = ImprintorNifWorld::new(config);

    match typst::compile(&world).output {
        Ok(document) => {
            let pdf_bytes = typst_pdf::pdf(&document, &PdfOptions::default()).unwrap();
            match std::fs::write(&output_path, pdf_bytes) {
                Ok(_) => Ok(output_path.into()),
                Err(err) => Err(err.to_string().into()),
            }
        }
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

rustler::init!("Elixir.Imprintor");
