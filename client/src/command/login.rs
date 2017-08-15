use std::net::{SocketAddr, IpAddr, Ipv4Addr};
use std::iter::FromIterator;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};

use rand::{Rng, OsRng};
use data_encoding::HEXUPPER;
use url::{Url, form_urlencoded};
use webbrowser;
use im::Map;

use futures::{Async, Poll};
use futures::task::{Task, current};
use futures::future::{self, Future, FutureResult};
use hyper::{self, StatusCode};
use hyper::server::{Http, Service, Request, Response};
use hyper::header::{ContentLength, ContentType};

use error::Error;
use config::{Config, Auth, get_config, write_config};

pub const USAGE: &'static str = "Login.

Usage:
    pm login [options]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {}


struct Done(Arc<AtomicBool>, Arc<Mutex<Option<Task>>>);

impl Done {
    fn new() -> Self {
        Done(Arc::new(AtomicBool::new(false)), Arc::new(Mutex::new(None)))
    }

    fn done(&self) {
        self.0.store(true, Ordering::Relaxed);
        match self.1.lock() {
            Ok(ref mutex) => {
                match **mutex {
                    Some(ref task) => {
                        task.notify();
                    }
                    None => (),
                }
            }
            _ => panic!("failed to acquire mutex!?"),
        }
    }
}

impl Future for Done {
    type Item = ();
    type Error = ();
    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        if self.0.load(Ordering::Relaxed) {
            Ok(Async::Ready(()))
        } else {
            match self.1.lock() {
                Ok(ref mut mutex) => {
                    **mutex = Some(current())
                }
                _ => panic!("failed to acquire mutex in poll!!11"),
            }
            Ok(Async::NotReady)
        }
    }
}

impl Clone for Done {
    fn clone(&self) -> Self {
        Done(self.0.clone(), self.1.clone())
    }
}




fn bad_request() -> Response {
    Response::new()
        .with_status(StatusCode::BadRequest)
        .with_header(ContentLength("400 Bad Request".len() as u64))
        .with_header(ContentType::plaintext())
        .with_body("400 Bad Request")
}

struct CallbackArgs {
    state: String,
    token: String,
}

fn parse_callback_args(req: &Request) -> Option<CallbackArgs> {
    req.uri().query().and_then(|query| {
        let q = Map::from_iter(form_urlencoded::parse(query.as_bytes()).into_owned());
        match (q.get(&"state".to_string()), q.get(&"token".to_string())) {
            (Some(ref state), Some(ref token)) if q.len() == 2 => Some(CallbackArgs {
                state: state.to_string(),
                token: token.to_string(),
            }),
            _ => None,
        }
    })
}

struct Callback {
    state: String,
    done: Done,
}

impl Service for Callback {
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;
    type Future = FutureResult<Response, hyper::Error>;

    fn call(&self, req: Request) -> Self::Future {
        if let Some(args) = parse_callback_args(&req) {
            if args.state != self.state {
                return future::ok(bad_request());
            }
            let out = format!("You authenticated! Your token is: {}", args.token);
            let config = get_config().expect("unable to read user config file");
            write_config(&Config {
                auth: Auth {
                    token: Some(args.token),
                    ..config.auth
                },
                ..config
            }).expect("unable to write user config file");
            self.done.done();
            future::ok(
                Response::new()
                    .with_header(ContentLength(out.len() as u64))
                    .with_header(ContentType::plaintext())
                    .with_body(out),
            )
        } else {
            future::ok(bad_request())
        }
    }
}

pub fn generate_secret() -> Result<String, Error> {
    let data: Vec<u8> = OsRng::new()?.gen_iter::<u8>().take(32).collect();
    Ok(HEXUPPER.encode(&data))
}

pub fn execute(_: Args) -> Result<(), Error> {
    let done = Done::new();
    let callback_done = done.clone();
    let token = generate_secret()?;
    let mut url = Url::parse("http://localhost:8000/login_client").unwrap();
    url.query_pairs_mut().append_pair("token", &token);
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let server = Http::new()
        .bind(&socket, move || {
            Ok(Callback {
                state: token.clone(),
                done: callback_done.clone(),
            })
        })
        .unwrap();

    url.query_pairs_mut().append_pair(
        "callback",
        &format!("http://{}", server.local_addr().unwrap()),
    );
    webbrowser::open(url.as_str())?;

    server.run_until(done).unwrap();
    Ok(())
}
