extern crate futures;
extern crate rix;
extern crate tokio;

use futures::{Future, Stream};

fn main() {
    let host = std::env::var("MATRIX_HOST").expect("Missing MATRIX_HOST");
    let token = std::env::var("MATRIX_TOKEN").expect("Missing MATRIX_TOKEN");

    let client = rix::Client::new(host, token).expect("Failed to construct client");

    tokio::run(
        client
            .whoami()
            .and_then(|name| {
                println!("{}", name);
                Ok(())
            })
            .map_err(|e| panic!("{:?}", e)),
    );
}
