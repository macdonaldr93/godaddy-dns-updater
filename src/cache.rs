use std::io::{Read, Write};
use std::fs;
use std::path::Path;
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct CacheContent {
    pub last_ip: String,
}

pub struct Cache {
    pub path: String,
}

impl Cache {
    pub fn read(&self) -> CacheContent {
        let path = Path::new(&self.path);

        if !path.exists() {
            &self.create();
        }

        let mut f = fs::File::open(path).expect("Unable to open file");
        let mut contents = String::new();
        f.read_to_string(&mut contents).expect("Unable to read file");
        let mut cache_content = CacheContent {
            last_ip: String::new(),
        };

        if contents.len() > 0 {
            cache_content = serde_json::from_str(&contents).unwrap();
        }

        cache_content
    }

    pub fn write(&self, cache_content: CacheContent) {
        let content = serde_json::to_string(&cache_content).unwrap();
        let mut f = &self.create();
        f.write_all(content.as_bytes()).expect("Unable to write data");
    }

    pub fn clear(&self) {
        let path = Path::new(&self.path);
        if path.exists() {
            fs::remove_file(path).unwrap();
        }
    }

    fn create(&self) -> fs::File {
        let path = Path::new(&self.path);
        fs::File::create(path).expect("Unable to create file")
    }
}
