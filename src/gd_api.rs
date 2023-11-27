use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug)]
pub struct Credentials {
    pub api_key: String,
    pub secret: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    pub kind: String,
    pub ip: String,
    pub domain: String,
    pub name: String,
    pub ttl: u64,
}

impl Record {
    pub fn hash(&self) -> u64 {
        let record_hash = [&self.kind, &self.domain, &self.name];
        let mut hasher = DefaultHasher::new();

        Hash::hash_slice(&record_hash, &mut hasher);
        hasher.finish()
    }

    pub async fn update(&self, credentials: &Credentials) -> bool {
        let url = format!(
            "https://api.godaddy.com/v1/domains/{}/records/{}/{}",
            &self.domain, &self.kind, &self.name
        );
        let client = reqwest::Client::new();
        let json = json!({
            "data": &self.ip,
            "name": &self.name,
            "ttl": &self.ttl,
            "type": &self.kind,
        })
        .to_string();
        let authorization = format!("sso-key {}:{}", credentials.api_key, credentials.secret);
        let result = client
            .put(url)
            .header("Authorization", authorization)
            .header("Content-Type", "application/json")
            .header("Content-Length", json.len())
            .body(json)
            .send()
            .await;

        match result {
            Ok(_) => {
                return true;
            }
            Err(_) => {
                return false;
            }
        }
    }
}
