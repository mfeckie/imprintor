use rustler::types::binary;
use rustler::{NifStruct, Term};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use typst::diag::{FileError, FileResult};
use typst::foundations::{Array, Dict, Str, Value};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};
use typst_kit::fonts::{FontSlot, Fonts};
use typst_pdf::PdfOptions;

#[derive(Debug)]
struct ImprintorWorld {
    source: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
    files: HashMap<FileId, Source>,
    time: time::OffsetDateTime,
}

impl ImprintorWorld {
    fn new(
        main_content: &str,
        json_data: Option<String>,
        extra_fonts: Option<Vec<String>>,
    ) -> Self {
        let font_searcher = match extra_fonts {
            Some(fonts) => Fonts::searcher()
                .include_system_fonts(true)
                .search_with(fonts),
            None => Fonts::searcher().include_system_fonts(true).search(),
        };

        let main = FileId::new(None, VirtualPath::new("/main.typ"));
        let mut files = HashMap::new();
        files.insert(main, Source::new(main, main_content.to_string()));

        // Create library with JSON data if provided
        let mut library = Library::default();

        if let Some(json_str) = json_data {
            if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&json_str) {
                let typst_value = json_to_typst_value(json_value);
                library.global.scope_mut().define("json_data", typst_value);
            }
        }

        Self {
            source: Source::detached(main_content),
            library: LazyHash::new(library),
            book: LazyHash::new(font_searcher.book),
            fonts: font_searcher.fonts,
            files,
            time: time::OffsetDateTime::now_utc(),
        }
    }
}

impl World for ImprintorWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> typst::diag::FileResult<Source> {
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| typst::diag::FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn file(&self, id: FileId) -> typst::diag::FileResult<Bytes> {
        use std::fs;
        use std::path::Path;
        let path = id.vpath().as_rootless_path();
        if let Ok(bytes) = fs::read(Path::new(&path)) {
            Ok(Bytes::new(bytes))
        } else {
            Err(typst::diag::FileError::NotFound(path.into()))
        }
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts[index].get()
    }

    fn today(&self, offset: Option<i64>) -> Option<Datetime> {
        let offset = offset.unwrap_or(0);
        let offset = time::UtcOffset::from_hms(offset.try_into().ok()?, 0, 0).ok()?;
        let time = self.time.checked_to_offset(offset)?;
        Some(Datetime::Date(time.date()))
    }
}

fn json_to_typst_value(json: serde_json::Value) -> typst::foundations::Value {
    use typst::foundations::{Array, Dict, IntoValue, Str, Value};

    match json {
        serde_json::Value::Null => Value::None,
        serde_json::Value::Bool(b) => b.into_value(),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i.into_value()
            } else if let Some(f) = n.as_f64() {
                f.into_value()
            } else {
                Value::None
            }
        }
        serde_json::Value::String(s) => Str::from(s).into_value(),
        serde_json::Value::Array(arr) => {
            let typst_array: Array = arr.into_iter().map(json_to_typst_value).collect();
            typst_array.into_value()
        }
        serde_json::Value::Object(obj) => {
            let mut dict = Dict::new();
            for (key, value) in obj {
                dict.insert(Str::from(key), json_to_typst_value(value));
            }
            dict.into_value()
        }
    }
}

#[rustler::nif]
fn compile_typst_to_pdf<'a>(
    env: rustler::Env<'a>,
    template: String,
    json_data: String,
    extra_fonts: Vec<String>,
) -> Result<rustler::Binary<'a>, String> {
    let world = ImprintorWorld::new(&template, Some(json_data), Some(extra_fonts));

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

/// A File that will be stored in the HashMap.
#[derive(Clone, Debug)]
struct FileEntry {
    bytes: Bytes,
    source: Option<Source>,
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
        let mut library = Library::default();

        if let Some(elixir_data) = config.data {
            let typst_value = typst_values_from_elxiir(elixir_data);
            library
                .global
                .scope_mut()
                .define("elixir_data", typst_value);
        }

        Self {
            source: Source::detached(config.source_document),
            fonts: font_searcher.fonts,
            time: time::OffsetDateTime::now_utc(),
            book: LazyHash::new(font_searcher.book),
            library: LazyHash::new(library),
            files: Arc::new(Mutex::new(HashMap::new())),
            root,
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
        let path = if let Some(_package) = id.package() {
            dbg!("CALLED in HERE");
            return Err(FileError::AccessDenied);
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
}

fn typst_values_from_elxiir(term: Term) -> typst::foundations::Value {
    match term.get_type() {
        rustler::TermType::Atom => {
            let atom: String = term.decode().unwrap();

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
                let key_str = key.atom_to_string().unwrap();
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

#[rustler::nif]
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

rustler::init!("Elixir.Imprintor");
