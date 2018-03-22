extern crate rix;
extern crate tokio_core;
extern crate futures;

use futures::Future;

fn main() {
    let host = std::env::var("MATRIX_HOST").expect("Missing MATRIX_HOST");
    let token = std::env::var("MATRIX_TOKEN").expect("Missing MATRIX_TOKEN");
    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let task = rix::client::sync(&host, &token, &handle, None)
        .and_then(|res| {
            rix::client::sync(&host, &token, &handle, Some(res.next_batch))
        });

    println!("{:?}", core.run(task).unwrap());
}
