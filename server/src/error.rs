use redis;
use medallion;
use rocket::http::Status;
use rocket::response::{Response, Responder};

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        Io(err: ::std::io::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Redis(err: redis::RedisError) {
            cause(err)
            description(err.description())
            from()
        }
        JWT(err: medallion::Error) {
            cause(err)
            description(err.description())
            from()
        }
        Status(code: Status) {}
    }
}

impl<'a> Responder<'a> for Error {
    fn respond(self) -> Result<Response<'a>, Status> {
        match self {
            Error::Status(code) => Err(code),
            // TODO real logging?
            _ => {
                println!("error: {:?}", &self);
                Err(Status::InternalServerError)
            }
        }
    }
}

pub type Res<A> = Result<A, Error>;
