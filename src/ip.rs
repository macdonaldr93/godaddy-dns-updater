use serde::Deserialize;

#[derive(Deserialize)]
struct HttpBinResponse {
    origin: String,
}

pub async fn current_ip() -> String {
    let uri = "http://httpbin.org/ip";
    let res = reqwest::get(uri)
        .await
        .expect("Failed to fetch IP")
        .json::<HttpBinResponse>()
        .await
        .expect("Failed to parse IP");

    return res.origin;
}
