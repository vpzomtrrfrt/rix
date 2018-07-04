use std;
use hyper;
use hyper_tls;
use serde_json;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        HTTP(e: hyper::Error) {
            from()
        }
        TLS(e: hyper_tls::Error) {
            from()
        }
        UTF8(e: std::string::FromUtf8Error) {
            from()
        }
        JSON(e: serde_json::Error) {
            from()
        }
        Other(details: String) {
            from()
        }
    }
}
