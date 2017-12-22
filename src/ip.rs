use std::io::{self};
use futures::{Future, Stream};
use hyper::Client;
use tokio_core::reactor::Core;
use serde_json;

pub fn current_ip() -> String {
    let mut core = Core::new().unwrap();
    let client = Client::new(&core.handle());

    let uri = "http://httpbin.org/ip".parse().unwrap();
    let work = client.get(uri)
        .and_then(|res| {
            let ip = res.body().concat2()
                .and_then(move |body| {
                    let v: serde_json::Value = serde_json::from_slice(&body).map_err(|e| {
                        io::Error::new(io::ErrorKind::Other, e)
                    })?;

                    Ok(v["origin"].as_str().unwrap().to_owned())
                });
            ip
        });

    core.run(work).unwrap()
}
