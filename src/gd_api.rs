use std::io::{self};
use futures::{Future, Stream};
use hyper::Client;
use hyper::{Method, Request};
use hyper::header::{Authorization, ContentLength, ContentType};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use serde_json;

pub struct Credentials {
    pub api_key: String,
    pub secret: String,
}

pub struct Record {
    pub kind: String,
    pub ip: String,
    pub domain: String,
    pub name: String,
    pub ttl: u64,
}

pub fn update_record(creds: &Credentials, record: &Record) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);
    let json = json!({
            "type": record.kind,
            "data": record.ip,
            "name": record.name,
            "ttl": record.ttl,
        }).to_string();
    let url = format!("https://api.godaddy.com/v1/domains/{}/records/{}/{}", record.domain, record.kind, record.name);
    let uri = url.parse().unwrap();

    println!("Updating {} with {}", url, json);

    let mut req = Request::new(Method::Put, uri);
    req.headers_mut().set(Authorization(format!("sso-key {}:{}", creds.api_key, creds.secret)));
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(json.len() as u64));
    req.set_body(json);

    let post = client.request(req).and_then(|res| {
        println!("GoDaddy API Status: {}", res.status());

        res.body().concat2()
            .and_then(move |body| {
                let v: serde_json::Value = serde_json::from_slice(&body).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, e)
                })?;

                println!("GoDaddy API Response: {}", v);

                Ok(())
            })
    });

    core.run(post).unwrap();
}
