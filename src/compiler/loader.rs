use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use syntax::codemap::{FileLoader, FileName};

pub struct ReplaceLoader {
    files: HashMap<FileName, String>,
}

// Custom file loader to replace files matching certain names for the codemap.
impl FileLoader for ReplaceLoader {
    fn file_exists(&self, path: &Path) -> bool {
        fs::metadata(path).is_ok()
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        let file = path.to_str().unwrap();
        if let Some(input) = self.files.get(file) {
            return Ok(input.clone());
        }

        let mut src = String::new();
        try!(File::open(path)).read_to_string(&mut src).map(move |_| src)
    }
}

impl ReplaceLoader {
    pub fn new() -> ReplaceLoader {
        ReplaceLoader { files: HashMap::new() }
    }

    pub fn add_file(&mut self, file: FileName, src: String) {
        self.files.insert(file, src);
    }
}
