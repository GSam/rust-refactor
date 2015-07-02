use std::collections::HashMap;
use std::path::Path;
use std::{fs};
use std::io::{self, Read};
use syntax::codemap::FileLoader;
use syntax::codemap::FileName;

pub struct ReplaceLoader {
    files: HashMap<FileName, String>
}

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
        try!(try!(fs::File::open(path)).read_to_string(&mut src));
        Ok(src)
    }
} 

impl ReplaceLoader {
    pub fn add_file(&mut self, file: FileName, src: String) {
        self.files.insert(file, src);
    }
}
