#[macro_use]
extern crate clap;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
#[macro_use]
extern crate serde_json;

use std::io::{self};
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

fn main() {
    let args = init_cli();

    // HTTP client
    let current_ip = fetch_current_ip();
    println!("Current IP address is {}", current_ip);

    // Post to GoDaddy
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);
    let json = json!({
            "type": args.record_type,
            "data": current_ip,
            "name": args.record_name,
            "ttl": args.record_ttl,
        }).to_string();
    let url = format!("https://api.godaddy.com/v1/domains/{}/records/{}/{}", args.domain, args.record_type, args.record_name);
    let uri = url.parse().unwrap();

    println!("Updating {} with {}", url, json);

    let mut req = Request::new(Method::Put, uri);
    req.headers_mut().set(Authorization(format!("sso-key {}:{}", args.api_key, args.api_key_secret)));
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
