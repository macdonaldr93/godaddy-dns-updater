use std::io::{Read, Write};
use std::fs;
use std::path::Path;
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub struct CacheContent {
    pub hash: u64,
    pub last_ip: String,
}

pub struct Cache<'a> {
    pub path: &'a Path,
}

impl<'a> Cache<'a> {
    pub fn read(&self) -> CacheContent {
        if !&self.path.exists() {
            self.create();
        }

        let mut f = fs::File::open(&self.path).expect("Unable to open file");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("Unable to read file");

        if contents.is_empty() {
            CacheContent {
                hash: 0,
                last_ip: String::new(),
            }
        } else {
            serde_json::from_str(&contents).unwrap()
        }
    }

    pub fn write(&self, cache_content: &CacheContent) {
        let content = serde_json::to_string(&cache_content).unwrap();
        let mut f = &self.create();
        f.write_all(content.as_bytes()).expect("Unable to write data");
    }

    pub fn clear(&self) {
        let compare = &true;
        if &self.path.exists() == compare {
            fs::remove_file(&self.path).unwrap();
        }
    }

    fn create(&self) -> fs::File {
        fs::File::create(&self.path).expect("Unable to create file")
    }
}
