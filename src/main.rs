#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate actix_web;
extern crate chrono;

use actix_web::{App,  http, HttpResponse, Json, server};
use chrono::{Local, Timelike};

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
}

fn main() {
    server::new(|| {
        App::new().resource("/next-arrival", |r| r.method(http::Method::POST)
            .with(next_arrival))
    }).bind("0.0.0.0:8000").expect("Address already in use").run();
    println!("app started on port 8000");
}

fn next_arrival(req: Json<NextArrivalRequest>) -> HttpResponse{
    let input = req.into_inner();
    let t = Local::now();
    let output = NextArrivalResponse{
        station: input.station,
        direction: input.direction,
        time: format!("{}:{}", t.hour() % 12, t.minute())
    };
    match serde_json::to_string(&output) {
        Ok(s) => return HttpResponse::Ok().content_type("application/json").body(s),
        _ => {
            println!("error forming json response from input received");
            return HttpResponse::BadRequest()
        },
    }
}

