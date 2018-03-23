extern crate rix;
extern crate tokio_core;
extern crate futures;
extern crate hyper;

use futures::{Future, Stream};
use std::str::FromStr;

fn main() {
    let host = std::env::var("MATRIX_HOST").expect("Missing MATRIX_HOST");
    let token = std::env::var("MATRIX_TOKEN").expect("Missing MATRIX_TOKEN");

    let mut core = tokio_core::reactor::Core::new().unwrap();
    let handle = core.handle();

    let task = rix::client::media::upload(&host, &token, &handle, hyper::mime::Mime::from_str("text/plain").unwrap(), "upload.rs", &include_bytes!("./upload.rs")[..])
        .and_then(|url| {
            rix::client::sync_stream(&host, &token, &handle)
                .skip(1)
                .for_each(move |frame| {
                    for evt in frame.events() {
                        if evt.event_type == "m.room.message" {
                            let body = evt.content["body"].as_str();
                            if let Some(body) = body {
                                if body == "!upload.rs" {
                                    if let Some(ref room) = evt.room {
                                        handle.spawn(rix::client::send_file(&host, &token, &handle, &room, &url, "upload.rs").map_err(|e| eprintln!("{:?}", e)));
                                    }
                                }
                            }
                        }
                    }
                    Ok(())
                })
        });

    core.run(task).unwrap();
}
