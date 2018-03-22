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
            for (room, content) in item.rooms.join {
                let events = content.timeline.events;
                for evt in events {
                    if evt.event_type == "m.room.message" {
                        let body = evt.content["body"].as_str();
                        if let Some(body) = body {
                            if body == "ping" {
                                handle.spawn(rix::client::send_message(&host, &token, &handle, &room, "pong").map_err(|e|eprintln!("{:?}", e)));
                            }
                        }
                    }
                }
            }
            Ok(())
        });

    core.run(task).unwrap();
}
