#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

extern crate actix_web;

use actix_web::{App, Error, HttpRequest, HttpResponse, Json, Responder, server};
use actix_web::Result as ActixResult;

#[derive(Deserialize)]
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

impl Responder for NextArrivalResponse{
    //https://actix.rs/api/actix-web/stable/actix_web/trait.Responder.html
    type Item = HttpResponse;
    type Error = Error;

    fn respond_to<S>(self, _req: &HttpRequest<S>) -> Result<HttpResponse, Error>{
        if self.e.is_some(){
            return Err(self.e.unwrap());
        } else {
            let body = serde_json::to_string(&self)?;
            Ok(HttpResponse::Ok()).content_type("application/json")
                .body(body)
        }
    }
}

fn main() {
    server::new(|| {
        App::new().resource("/next-arrival", |r| r.method(http::Method::Post)
            .with(next_arrival))
    }).bind("0.0.0.0:8000").unwrap().run();
    println!("app started on port 8000");
}

//fn next_arrival(req: Json<NextArrivalRequest>) -> impl Responder{
fn next_arrival(req: Json<NextArrivalRequest>) -> ActixResult<String>{
    //TODO: validate direction and station strings, get current time and use that too.

    //for now, just print out what we get

    return Ok(HttpResponse::Ok().content_type())
}

