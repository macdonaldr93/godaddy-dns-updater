extern crate clap;
extern crate futures;
extern crate serde;

use clap::{Parser, Subcommand};
use std::path::Path;

mod cache;
mod gd_api;
mod ip;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Print out additional debugging logs
    #[arg(short = 'v', long = "verbose")]
    verbose: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Update a DNS record to the current or given IP address
    Update {
        /// The API key from your GoDaddy account
        #[arg(short = 'a', long = "api-key")]
        api_key: String,
        /// The API secret from your GoDaddy account
        #[arg(short = 's', long = "secret")]
        secret: String,
        /// The domain of the DNS record
        #[arg(short = 'd', long = "domain")]
        domain: String,
        /// The kind of DNS record (default: A)
        #[arg(short = 'k', long = "kind", default_value = "A")]
        kind: String,
        /// The name of the DNS record
        #[arg(short = 'n', long = "name")]
        name: String,
        /// The time-to-live of the DNS record in seconds
        #[arg(short = None, long = "ttl", default_value_t = 600)]
        ttl: u64,
        /// The IP address
        #[arg(short = None, long = "ip")]
        ip: Option<String>,
    },
    /// Resets cache for the last IP address
    Reset {},
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cache = cache::Cache {
        path: Path::new("./godaddy-dns-updater.json"),
    };

    match &cli.command {
        Some(Commands::Update {
            api_key,
            domain,
            ip,
            kind,
            name,
            secret,
            ttl,
        }) => {
            let ip = match ip {
                Some(ip) => ip.to_owned(),
                _ => ip::current_ip().await,
            };

            println!("Updating DNS record {} to {}", name, ip);

            let credentials = gd_api::Credentials {
                api_key: api_key.to_owned(),
                secret: secret.to_owned(),
            };
            let record = gd_api::Record {
                kind: kind.to_owned(),
                ip: ip.to_owned(),
                domain: domain.to_owned(),
                name: name.to_owned(),
                ttl: ttl.to_owned(),
            };
            let record_hash = record.hash();

            let mut cache_content = cache.read();

            if cli.verbose {
                println!("Restoring cache {:?}", cache_content);
            }

            if record_hash == cache_content.hash && ip == cache_content.last_ip {
                println!("DNS Record and IP are the same");
                println!("✅ Done");
                return;
            }

            cache_content = cache::CacheContent {
                hash: record_hash,
                last_ip: ip.to_owned(),
            };

            if cli.verbose {
                println!("Caching content {:?}", cache_content);
            }

            cache.write(&cache_content);
            record.update(&credentials).await;
        }
        Some(Commands::Reset { .. }) => {
            cache.clear();
            println!("✅ Cache reset");
        }
        None => {}
    }
}
