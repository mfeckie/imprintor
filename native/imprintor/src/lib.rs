use std::collections::HashMap;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};
use typst_kit::fonts::{FontSlot, Fonts};
use typst_pdf::PdfOptions;

#[derive(Debug)]
struct TypstWorld {
    source: Source,
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
    files: HashMap<FileId, Source>,
    time: time::OffsetDateTime,
}

impl TypstWorld {
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

impl World for TypstWorld {
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
        Err(typst::diag::FileError::NotFound(
            id.vpath().as_rootless_path().into(),
        ))
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
    let world = TypstWorld::new(&template, Some(json_data), Some(extra_fonts));

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
