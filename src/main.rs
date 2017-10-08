extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate hyper_tls;
extern crate tokio_core;
#[macro_use]
extern crate error_chain;

use std::io::{self, Read};
use futures::{Future, Stream};
use hyper::{Client, Method};
use hyper::client::Request;
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Core;
use serde_json::Value;
use std::process::Command;

mod errors {
    use super::serde_json;
    use super::hyper;

    error_chain! {
        foreign_links {
            Http(hyper::Error);
            Serde(serde_json::Error);
        }
    }
}

pub use errors::*;

fn main() {
    
    let mut input = String::new();
    let _ = io::stdin().read_to_string(&mut input).unwrap();
    
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, &handle).unwrap())
        .build(&handle);

    let uri = "https://www.hastebin.com/documents".parse().unwrap();

    let mut req = Request::new(Method::Post, uri);
    req.set_body(input);

    let post = client.request(req)
        .then(|r| r.chain_err(|| "Unable to make https request"))
        .and_then(|res| res.body().concat2().then(|r| r.chain_err(|| "Unable to concat response's body")))
        .and_then(move |body| serde_json::from_slice::<Value>(&body).chain_err(|| "Unable to parse response's body"));

    let value: Value = core.run(post).unwrap();
    let hastebin = format!("https://hastebin.com/{}", value["key"].to_string().replace("\"", ""));
    println!("👌  Uploaded on hastebin at {}", hastebin);
    Command::new("open").arg(hastebin).spawn().expect("Failed to open in your browser.");
}
