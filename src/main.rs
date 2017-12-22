#[macro_use]
extern crate clap;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::fs::OpenOptions;
use clap::{App, Arg};
use futures::{Future, Stream};
use hyper::Client;
use hyper::{Method, Request};
use hyper::header::{Authorization, ContentLength, ContentType};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use serde_json::Value;

struct CliArgs {
    api_key: String,
    api_key_secret: String,
    domain: String,
    record_name: String,
    record_type: String,
    record_ttl: u64,
}

#[derive(Serialize, Deserialize)]
struct CacheContent {
    last_ip: String,
}

struct Cache {
    path: String,
}

impl Cache {
    fn read_cache(&self) -> CacheContent {
        let path = Path::new(&self.path);
        let mut f = OpenOptions::new()
                        .read(path.exists())
                        .write(!path.exists())
                        .create_new(!path.exists())
                        .open(path)
                        .expect("Unable to open file");
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

    fn write_cache(&self, cache_content: CacheContent) {
        let path = Path::new(&self.path);
        let content = serde_json::to_string(&cache_content).unwrap();
        let mut f = File::create(path).expect("Unable to create file");
        f.write_all(content.as_bytes()).expect("Unable to write data");
    }
}

fn main() {
    let args = init_cli();
    let cache = Cache { path: String::from("./godaddy-dns-updater.json") };

    // HTTP client
    let current_ip = fetch_current_ip();
    println!("Current IP address is {}", current_ip);

    // Check cache
    let cache_content = cache.read_cache();
    println!("Current cached IP address is {}", cache_content.last_ip);

    if current_ip == cache_content.last_ip {
        println!("IP address is the same. Exiting...");
        return;
    }

    let cache_updated = CacheContent {
        last_ip: current_ip.to_owned(),
    };
    println!("Cached IP address updated to {}", cache_content.last_ip);
    cache.write_cache(cache_updated);

    // Post to GoDaddy
    update_record(current_ip, args);
}

fn init_cli() -> CliArgs {
    let matches = App::new("GoDaddy DNS Updater")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Update GoDaddy DNS records")
        .args(&[
            Arg::with_name("api_key")
                .value_name("KEY")
                .help("sets the API key for your GoDaddy account")
                .required(true)
                .takes_value(true)
                .short("a")
                .long("apiKey"),
            Arg::with_name("api_key_secret")
                .value_name("SECRET")
                .help("sets the API key secret for your GoDaddy account")
                .required(true)
                .takes_value(true)
                .short("s")
                .long("secret"),
            Arg::with_name("domain")
                .help("sets the domain to update DNS records")
                .required(true)
                .takes_value(true)
                .short("d")
                .long("domain"),
            Arg::with_name("record_name")
                .help("sets the name of the record")
                .required(true)
                .takes_value(true)
                .short("n")
                .long("name"),
            Arg::with_name("record_type")
                .help("sets the type of the record")
                .default_value("A")
                .required(true)
                .takes_value(true)
                .short("t")
                .long("type"),
            Arg::with_name("record_ttl")
                .help("sets the time to live of the record in seconds")
                .default_value("600")
                .takes_value(true)
                .short("l")
                .long("ttl"),
        ])
        .get_matches();

    // Get CLI information for request
    let cli_args = CliArgs {
        api_key: matches.value_of("api_key").unwrap().to_owned(),
        api_key_secret: matches.value_of("api_key_secret").unwrap().to_owned(),
        domain: matches.value_of("domain").unwrap().to_owned(),
        record_name: matches.value_of("record_name").unwrap().to_owned(),
        record_type: matches.value_of("record_type").unwrap().to_owned(),
        record_ttl: value_t!(matches, "record_ttl", u64).unwrap().to_owned()
    };

    cli_args
}

fn fetch_current_ip() -> String {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let uri = "http://httpbin.org/ip".parse().unwrap();
    let work = client.get(uri)
        .and_then(|res| {
            let ip = res.body().concat2()
                .and_then(move |body| {
                    let v: Value = serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;

                    Ok(v["origin"].as_str().unwrap().to_owned())
                });
            ip
        });

    core.run(work).unwrap()
}

fn update_record(current_ip: String, cli_args: CliArgs) {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);
    let json = json!({
            "type": cli_args.record_type,
            "data": current_ip,
            "name": cli_args.record_name,
            "ttl": cli_args.record_ttl,
        }).to_string();
    let url = format!("https://api.godaddy.com/v1/domains/{}/records/{}/{}", cli_args.domain, cli_args.record_type, cli_args.record_name);
    let uri = url.parse().unwrap();

    println!("Updating {} with {}", url, json);

    let mut req = Request::new(Method::Put, uri);
    req.headers_mut().set(Authorization(format!("sso-key {}:{}", cli_args.api_key, cli_args.api_key_secret)));
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(json.len() as u64));
    req.set_body(json);

    let post = client.request(req).and_then(|res| {
        println!("GoDaddy API Status: {}", res.status());

        res.body().concat2()
            .and_then(move |body| {
                let v: Value = serde_json::from_slice(&body).map_err(|e| {
                    io::Error::new(io::ErrorKind::Other, e)
                })?;

                println!("GoDaddy API Response: {}", v);

                Ok(())
            })
    });

    core.run(post).unwrap();
}
