#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate actix_web;

use actix_web::{App,  http, HttpResponse, Json, server};

#[derive(Serialize, Deserialize)]
struct NextArrivalRequest{
    station: String,
    direction: String,
}

#[derive(Serialize)]
struct NextArrivalResponse{
    station: String,
    direction: String,
    time: String,
    //TODO: Give time as in 00:00pm or "6 minutes"
}

fn main() {
    server::new(|| {
        App::new().resource("/next-arrival", |r| r.method(http::Method::POST)
            .with(next_arrival))
    }).bind("0.0.0.0:8000").unwrap().run();
    println!("app started on port 8000");
}

fn next_arrival(req: Json<NextArrivalRequest>) -> HttpResponse{
    //TODO: validate direction and station strings, get current time and use that too.

    //for now, just print out what we get

    match serde_json::to_string(&req.into_inner()) {
        Ok(s) => return HttpResponse::Ok().content_type("application/json").body(s),
        _ => return HttpResponse::BadRequest().content_type("application/json").body("error decoding"),
    }

}

