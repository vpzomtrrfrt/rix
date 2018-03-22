extern crate rix;
extern crate tokio_core;
extern crate futures;

use futures::{Future, Stream};

fn main() {
    let host = std::env::var("MATRIX_HOST").expect("Missing MATRIX_HOST");
    let token = std::env::var("MATRIX_TOKEN").expect("Missing MATRIX_TOKEN");
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let task = rix::client::sync_stream(&host, &token, &handle)
        .skip(1)
        .for_each(|item| {
            println!("{:?}", item);
            Ok(())
        });

    core.run(task).unwrap();
}
