extern crate hyper;
extern crate hyper_tls;
extern crate futures;
extern crate rand;
extern crate urlencoding;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate quick_error;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate try_future;

pub mod client;
pub mod error;

type HttpsClient = hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>;

pub use client::Client;
pub use error::Error;
