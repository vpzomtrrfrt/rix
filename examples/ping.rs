extern crate rix;
extern crate futures;
extern crate tokio;

use futures::{Future, Stream};

fn main() {
    let host = std::env::var("MATRIX_HOST").expect("Missing MATRIX_HOST");
    let token = std::env::var("MATRIX_TOKEN").expect("Missing MATRIX_TOKEN");

    let task = rix::client::sync_stream(&host, &token)
        .skip(1)
        .for_each(move |item| {
            println!("{:?}", item);
            for evt in item.events() {
                if evt.event_type == "m.room.message" {
                    let body = evt.content["body"].as_str();
                    if let Some(body) = body {
                        if body == "ping" {
                            if let Some(ref room) = evt.room {
                                tokio::spawn(rix::client::send_message(&host, &token, &room, "pong").map_err(|e|eprintln!("{:?}", e)));
                            }
                        }
                    }
                }
            }
            Ok(())
        })
    .map_err(|e| {
        eprintln!("{:?}", e);
    });

    tokio::run(task);
}
