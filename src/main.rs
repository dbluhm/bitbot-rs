extern crate futures;
extern crate hyper;
extern crate hyper_tls;
extern crate tokio_core;
extern crate failure;
extern crate toml;
extern crate serde_json;

#[macro_use]
extern crate failure_derive;

use failure::Error;
use std::path::Path;

#[derive(Fail, Debug)]
#[fail(display = "Request Error: {}", res)]
struct RequestError {
    res: hyper::StatusCode
}

#[derive(Fail, Debug)]
enum TomlError {
    #[fail(display = "Toml Element \"{}\" could not be found", name)]
    InvalidStructure {
        name: String,
    },
    #[fail(display = "Toml Element \"{}\" is invalid", name)]
    InvalidValue {
        name: String,
    }
}

struct Conf {
    endpoint: String,
    function: String,
    token: String,
    chat_id: String,
    markdown: Option<bool>
}

fn main() {
    use std::io::Write;
    use std::env::args;
    use std::io::{self, Read};

    let argv: Vec<String> = args().collect();
    let conf_path = if argv.len() > 1 {
        &argv[1]
    } else {
        "conf.toml"
    };

    let mut message = String::new();
    let stdin = io::stdin();
    let mut handle = stdin.lock();

    if let Err(err) = handle.read_to_string(&mut message) {
        writeln!(std::io::stderr(), "could not read from stdin: {}", err).unwrap();
        ::std::process::exit(1);
    }
    ::std::process::exit(match send_message(conf_path, &message) {
        Ok(_) => 0,
        Err(err) => {
            writeln!(std::io::stderr(), "failed: {}", err).unwrap();
            1
        }
    });
}

fn parse_conf<P: AsRef<Path>>(path_to_conf: P) -> Result<Conf, Error> {
    use std::fs::File;
    use std::io::Read;
    use toml::Value;

    let mut string = String::new();
    File::open(path_to_conf)?.read_to_string(&mut string)?;

    let toml = string.parse::<Value>()?;

    let conf = Conf{
        endpoint: toml.get("endpoint")
            .ok_or(TomlError::InvalidStructure{name:"endpoint".to_string()})?
            .as_str()
            .ok_or(TomlError::InvalidValue{name:"endpoint".to_string()})?
            .to_string(),
        function: toml.get("function")
            .ok_or(TomlError::InvalidStructure{name:"function".to_string()})?
            .as_str()
            .ok_or(TomlError::InvalidValue{name:"function".to_string()})?
            .to_string(),
        token: toml.get("token")
            .ok_or(TomlError::InvalidStructure{name:"token".to_string()})?
            .as_str()
            .ok_or(TomlError::InvalidValue{name:"token".to_string()})?
            .to_string(),
        chat_id: toml.get("chat_id")
            .ok_or(TomlError::InvalidStructure{name:"chat_id".to_string()})?
            .as_str()
            .ok_or(TomlError::InvalidValue{name:"chat_id".to_string()})?
            .to_string(),
        markdown: toml.get("markdown")
            .and_then(|val| val.as_bool())
    };

    Ok(conf)
}

fn send_message(path_to_conf: &str, message: &str) -> Result<(), Error> {
    use hyper::Client;
    use hyper_tls::HttpsConnector;
    use tokio_core::reactor::Core;

    let conf = parse_conf(path_to_conf)?;

    let mut core = Core::new()?;
    let handle = core.handle();
    let client = Client::configure()
        .connector(HttpsConnector::new(2, &handle)?)
        .build(&handle);

    let req = build_request(
        &conf,
        message
    )?;

    let res = core.run(client.request(req))?;
    let status = res.status();
    if !status.is_success() {
        return Err(Error::from(RequestError{res: status}));
    }
    Ok(())
}

fn build_request(conf: &Conf, text: &str) -> Result<hyper::Request, Error> {
    use hyper::{Method, Request};
    use hyper::header::{ContentLength, ContentType};

    let uri = format!("{}{}/{}", conf.endpoint, conf.token, conf.function).parse()?;

    let mut m = serde_json::Map::new();
    m.insert("chat_id".to_string(), conf.chat_id.clone().into());
    m.insert("text".to_string(), text.into());
    if let Some(flag) = conf.markdown {
        if flag {
            m.insert("parse_mode".to_string(), "Markdown".into());
        }
    }

    let json: serde_json::Value = m.into();
    let json = json.to_string();

    let mut req = Request::new(Method::Post, uri);
    req.headers_mut().set(ContentType::json());
    req.headers_mut().set(ContentLength(json.len() as u64));
    req.set_body(json);

    Ok(req)
}
