#[macro_use] extern crate clap;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate serde;
#[macro_use] extern crate serde_json;
#[macro_use] extern crate serde_derive;

use std::io::{self, Write};
use clap::{App, Arg, SubCommand};
use std::path::Path;

mod cache;
mod ip;
mod gd_api;

fn main() {
    let app = cli();
    let mut help_buffer = Vec::new();
    app.write_help(&mut help_buffer).unwrap();

    let matches = app.get_matches();
    let cache = cache::Cache { path: Path::new("./godaddy-dns-updater.json") };

    match matches.subcommand() {
        ("cache:clear", Some(_)) => {
            cache.clear();
            println!("Cache cleared");
        },
        ("update", Some(update_matches)) => {
            let ip_address = match update_matches.value_of("ip_address") {
                Some(ip) => ip.to_owned(),
                _ => ip::current_ip()
            };

            println!("IP address is {}", ip_address);

            let credentials = gd_api::Credentials {
                api_key: update_matches.value_of("api_key").unwrap().to_owned(),
                secret: update_matches.value_of("api_key_secret").unwrap().to_owned(),
            };
            let record = gd_api::Record {
                kind: update_matches.value_of("record_type").unwrap().to_owned(),
                ip: ip_address.to_owned(),
                domain: update_matches.value_of("domain").unwrap().to_owned(),
                name: update_matches.value_of("record_name").unwrap().to_owned(),
                ttl: value_t!(update_matches, "record_ttl", u64).unwrap().to_owned(),
            };
            let record_hash = record.hash();

            let mut cache_content = cache.read();
            println!("Cached record is {:?}", cache_content);

            if record_hash == cache_content.hash && ip_address == cache_content.last_ip {
                println!("Record and IP address are the same. Exiting...");
                return;
            }

            cache_content = cache::CacheContent {
                hash: record_hash,
                last_ip: ip_address.to_owned(),
            };
            println!("Cached record updated to {:?}", cache_content);

            cache.write(&cache_content);
            gd_api::update_record(&credentials, &record);
        },
        ("", None)   => {
            let stdout = io::stdout();
            let mut handle = stdout.lock();
            handle.write_all(&help_buffer).unwrap();
        },
        _            => unreachable!(),
    }
}

fn cli() -> clap::App<'static, 'static> {
    App::new("GoDaddy DNS Updater")
        .version(crate_version!())
        .author(crate_authors!())
        .about("Update GoDaddy DNS records")
        .subcommand(SubCommand::with_name("update")
            .about("Updates GoDaddy DNS record with IP address")
            .aliases(&["u"])
            .args(&[
                Arg::with_name("ip_address")
                    .help("sets the IP address to update DNS records")
                    .takes_value(true)
                    .short("i")
                    .long("ip"),
                Arg::with_name("api_key")
                    .help("sets the API key for your GoDaddy account")
                    .required(true)
                    .takes_value(true)
                    .short("a")
                    .long("api_key"),
                Arg::with_name("api_key_secret")
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
        )
        .subcommand(SubCommand::with_name("cache:clear")
            .about("Clears last IP address cache")
        )
}
