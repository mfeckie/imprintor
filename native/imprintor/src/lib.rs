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
    fn new(main_content: &str) -> Self {
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

        Self {
            library: Prehashed::new(Library::default()),
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

#[rustler::nif]
fn compile_typst_to_pdf<'a>(
    env: rustler::Env<'a>,
    template: String,
    data: HashMap<String, String>,
) -> Result<rustler::Binary<'a>, String> {
    // Replace placeholders in template with actual data
    let mut content = template;
    for (key, value) in data {
        let placeholder = format!("{{{{{}}}}}", key);
        content = content.replace(&placeholder, &value);
    }

    let world = TypstWorld::new(&content);

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
