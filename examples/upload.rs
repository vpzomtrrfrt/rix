extern crate futures;
extern crate rix;
extern crate tokio;

use futures::{Future, Stream};

fn main() {
    let host = std::env::var("MATRIX_HOST").expect("Missing MATRIX_HOST");
    let token = std::env::var("MATRIX_TOKEN").expect("Missing MATRIX_TOKEN");

    let client = rix::Client::new(host, token).expect("Failed to construct client");

    let task = client
        .upload_media(
            "text/plain",
            "upload.rs",
            &include_bytes!("./upload.rs")[..],
        )
        .and_then(|url| {
            client.sync_stream().skip(1).for_each(move |frame| {
                for evt in frame.events() {
                    if evt.event_type == "m.room.message" {
                        let body = evt.content["body"].as_str();
                        if let Some(body) = body {
                            if body == "!upload.rs" {
                                if let Some(ref room) = evt.room {
                                    tokio::spawn(
                                        client
                                            .send_file(&room, &url, "upload.rs")
                                            .map_err(|e| eprintln!("{:?}", e)),
                                    );
                                }
                            }
                        }
                    }
                }
                Ok(())
            })
        })
        .map_err(|e| {
            eprintln!("{:?}", e);
        });

    tokio::run(task);
}
