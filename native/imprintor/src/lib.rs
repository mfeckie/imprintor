use comemo::Prehashed;
use fontdb::Database;
use std::collections::HashMap;
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source, VirtualPath};
use typst::text::{Font, FontBook};
use typst::{Library, World};

#[derive(Debug)]
struct TypstWorld {
    library: Prehashed<Library>,
    book: Prehashed<FontBook>,
    fonts: Vec<Font>,
    files: HashMap<FileId, Source>,
    main: FileId,
}

impl TypstWorld {
    fn new(main_content: &str, json_data: Option<String>) -> Self {
        let mut db = Database::new();
        db.load_system_fonts();

        let mut fonts = Vec::new();
        let mut book = FontBook::new();

        // Load fewer fonts to speed up initialization
        let mut font_count = 0;
        for face in db.faces() {
            if font_count >= 50 {
                // Limit to first 50 fonts
                break;
            }

            let data = match &face.source {
                fontdb::Source::Binary(data) => data.as_ref().as_ref(),
                fontdb::Source::File(path) => {
                    if let Ok(data) = std::fs::read(path) {
                        Box::leak(data.into_boxed_slice())
                    } else {
                        continue;
                    }
                }
                _ => continue,
            };

            if let Some(font) = Font::new(Bytes::from(data), face.index) {
                book.push(font.info().clone());
                fonts.push(font);
                font_count += 1;
            }
        }

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
            library: Prehashed::new(library),
            book: Prehashed::new(book),
            fonts,
            files,
            main,
        }
    }
}

impl World for TypstWorld {
    fn library(&self) -> &Prehashed<Library> {
        &self.library
    }

    fn book(&self) -> &Prehashed<FontBook> {
        &self.book
    }

    fn main(&self) -> Source {
        self.source(self.main).unwrap()
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
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Some(Datetime::from_ymd(2024, 1, 1).unwrap())
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
) -> Result<rustler::Binary<'a>, String> {
    let world = TypstWorld::new(&template, Some(json_data));

    match typst::compile(&world, &mut Default::default()) {
        Ok(document) => {
            let pdf_bytes = typst_pdf::pdf(&document, typst::foundations::Smart::Auto, None);
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
