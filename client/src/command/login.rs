use std::iter::FromIterator;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::thread;

use data_encoding::HEXUPPER;
use failure;
use hyper::body::Body;
use hyper::header::{CONTENT_LENGTH, CONTENT_TYPE};
use hyper::server::Server;
use hyper::service::{NewService, Service};
use hyper::{Error, Request, Response, StatusCode};
use crate::im::OrdMap as Map;
use mime;
use rand::prelude::random;
use tokio::prelude::future::{ok, Future, FutureResult};
use tokio::prelude::task::{current, Task};
use tokio::prelude::{Async, Poll};
use tokio::runtime::Runtime;
use url::{form_urlencoded, Url};
use webbrowser;

use crate::config::{write_config, Auth, Config};

pub const USAGE: &str = "Login.

Usage:
    pm login [options]

Options:
    -h, --help     Display this message.
";

#[derive(Debug, Deserialize)]
pub struct Args {}

const AUTHENTICATED_DOC: &str = "
<html>
  <head>
    <style>
      body { text-align: center; background: white; }
    </style>
  </head>
  <body>
    <h1>You are authenticated!</h1>
    <p>You may safely close this window.</p>
  </body>
</html>
";

#[derive(Clone)]
struct Done<A> {
    value: Arc<Mutex<Option<A>>>,
    task: Arc<Mutex<Option<Task>>>,
}

impl<A> Done<A> {
    fn new() -> Self {
        Done {
            value: Arc::new(Mutex::new(None)),
            task: Arc::new(Mutex::new(None)),
        }
    }

    fn done(&self, value: A) {
        match self.value.lock() {
            Ok(ref mut mutex) => **mutex = Some(value),
            _ => panic!("failed to acquire mutex!?"),
        }
        match self.task.lock() {
            Ok(ref mutex) => if let Some(ref task) = **mutex {
                task.notify();
            },
            _ => panic!("failed to acquire mutex!?"),
        }
    }
}

impl<A: Clone> Done<A> {
    fn get(&self) -> Option<A> {
        match self.value.lock() {
            Ok(ref mutex) => (**mutex).clone(),
            _ => panic!("failed to acquire mutex!?"),
        }
    }
}

impl<A> Future for Done<A> {
    type Item = ();
    type Error = ();

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match self.value.lock() {
            Ok(ref mutex) => {
                if (**mutex).is_some() {
                    Ok(Async::Ready(()))
                } else {
                    match self.task.lock() {
                        Ok(ref mut mutex) => **mutex = Some(current()),
                        _ => panic!("failed to acquire mutex in poll!!11"),
                    }
                    Ok(Async::NotReady)
                }
            }
            _ => panic!("failed to acquire mutex in poll!!11"),
        }
    }
}

fn bad_request() -> Response<Body> {
    let mut response = Response::builder();
    response
        .header(CONTENT_LENGTH, "400 Bad Request".len())
        .status(StatusCode::BAD_REQUEST);
    response.body(Body::from("400 Bad Request")).unwrap()
}

struct CallbackArgs {
    state: String,
    token: String,
}

fn parse_callback_args<Whatever>(req: &Request<Whatever>) -> Option<CallbackArgs> {
    req.uri().query().and_then(|query| {
        let mut q =
            Map::<String, String>::from_iter(form_urlencoded::parse(query.as_bytes()).into_owned());
        match (q.remove("state"), q.remove("token")) {
            (Some(state), Some(token)) => Some(CallbackArgs { state, token }),
            _ => None,
        }
    })
}

pub fn generate_secret() -> String {
    let data = random::<[u8; 32]>();
    HEXUPPER.encode(&data)
}

struct Callback {
    state: String,
    done: Done<String>,
}

impl NewService for Callback {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Service = CallbackService;
    type InitError = Error;
    type Future = FutureResult<CallbackService, Error>;

    fn new_service(&self) -> Self::Future {
        ok(CallbackService {
            state: self.state.clone(),
            done: self.done.clone(),
        })
    }
}

struct CallbackService {
    state: String,
    done: Done<String>,
}

impl Service for CallbackService {
    type ReqBody = Body;
    type ResBody = Body;
    type Error = Error;
    type Future = FutureResult<Response<Body>, Error>;

    fn call(&mut self, req: Request<Self::ReqBody>) -> Self::Future {
        if let Some(args) = parse_callback_args(&req) {
            if args.state != self.state {
                return ok(bad_request());
            }
            self.done.done(args.token);
            let mut response = Response::builder();
            response.header(CONTENT_LENGTH, AUTHENTICATED_DOC.len());
            response.header(CONTENT_TYPE, mime::TEXT_HTML.as_ref());
            ok(response.body(Body::from(AUTHENTICATED_DOC)).unwrap())
        } else {
            ok(bad_request())
        }
    }
}

pub fn execute(_: Args) -> Result<(), failure::Error> {
    // TODO we need to ensure that attackers cannot inject their own token by
    // connecting to our callback service via iframe
    let done = Done::new();
    let secret = generate_secret();
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);
    let service = Callback {
        state: secret.clone(),
        done: done.clone(),
    };
    let server = Server::bind(&socket).serve(service);

    // TODO This is the URL for the registry server, it should not be localhost.
    let mut url = Url::parse("http://localhost:8000/login_client").unwrap();
    url.query_pairs_mut().append_pair("token", &secret);
    url.query_pairs_mut()
        .append_pair("callback", &format!("http://{}", server.local_addr()));

    let mut rt = Runtime::new().unwrap();
    rt.spawn(
        server
            .with_graceful_shutdown(done.clone())
            .map_err(|e| eprintln!("HTTP server error: {}", e)),
    );
    thread::spawn(move || webbrowser::open(url.as_str()));
    rt.shutdown_on_idle().wait().unwrap();

    let token = done
        .get()
        .expect("unable to get auth token from web server");

    // let config = get_config()?;
    write_config(&Config {
        auth: Auth { token: Some(token) },
    })?;

    Ok(())
}
