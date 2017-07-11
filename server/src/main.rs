#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
extern crate rocket_contrib;

use rocket_contrib::JSON;

#[derive(Serialize)]
struct Message {
    action: String,
    data: String
}

#[get("/lol/<action>")]
fn hello(action: &str) -> JSON<Message> {
    JSON(Message {
        action: action.to_string(),
        data: "rofl".to_string()
    })
}

fn main() {
    rocket::ignite().mount("/", routes![hello]).launch()
}
