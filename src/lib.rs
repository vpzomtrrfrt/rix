extern crate hyper;
extern crate hyper_tls;
extern crate futures;
extern crate tokio_core;
extern crate rand;
extern crate urlencoding;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;

macro_rules! box_fut_try(
    ($e:expr) => (match $e
                  {
                      Ok(e) => e,
                      Err(err) => return Box::new(futures::future::err(err))
                  })
    );

pub mod client;
pub mod error;
