#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate actix_web;
extern crate chrono;
extern crate csv;

use actix_web::{App,  http, HttpResponse, Json, server};
use chrono::{Datelike, DateTime, Local, Timelike, Weekday};
use std::error::Error;

#[derive(RustEmbed)]
#[folder = "data/"]
struct Asset;

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

#[derive(Debug, Deserialize, Serialize)]
struct StationTimeSlice{
    #[serde(rename = "Lambert Airport Terminal #1")]
    lambert_t1: Option<String>,
    #[serde(rename = "Lambert Airport Terminal #2")]
    lambert_t2: Option<String>,
    #[serde(rename = "North Hanley Station")]
    north_hanley: Option<String>,
    #[serde(rename = "UMSL North Station")]
    umsl_north: Option<String>,
    #[serde(rename = "UMSL South Station")]
    umsl_south: Option<String>,
    #[serde(rename = "Rock Road Station")]
    rock_road: Option<String>,
    #[serde(rename = "Wellston Station")]
    wellston: Option<String>,
    #[serde(rename = "Delmar Loop Station")]
    delmar_loop: Option<String>,
    #[serde(rename = "Shrewsbury Station")]
    shrewsbury: Option<String>,
    #[serde(rename = "Sunnen Station")]
    sunnen: Option<String>,
    #[serde(rename = "MaplewoodManchester Station")]
    maplewood_manchester: Option<String>,
    #[serde(rename = "Brentwood Station")]
    brentwood: Option<String>,
    #[serde(rename = "Richmond Heights Station")]
    richmond_heights: Option<String>,
    #[serde(rename = "Clayton Station")]
    clayton: Option<String>,
    #[serde(rename = "Forsyth Station")]
    forsyth: Option<String>,
    #[serde(rename = "U City Big Bend Station")]
    u_city: Option<String>,
    #[serde(rename = "Skinker Station")]
    skinker: Option<String>,
    #[serde(rename = "Forest Park DeBaliviere Station")]
    forest_park: Option<String>,
    #[serde(rename = "Central West End Station")]
    cwe: Option<String>,
    #[serde(rename = "Cortex Station")]
    cortex: Option<String>,
    #[serde(rename = "Grand Station")]
    grand: Option<String>,
    #[serde(rename = "Union Station")]
    union: Option<String>,
    #[serde(rename = "Civic Center Station")]
    civic_center: Option<String>,
    #[serde(rename = "Stadium Station")]
    stadium: Option<String>,
    #[serde(rename = "8th & Pine Station")]
    eight_pine: Option<String>,
    #[serde(rename = "Convention Center Station")]
    convention_center: Option<String>,
    #[serde(rename = "Lacledes Landing Station")]
    lacledes_landing: Option<String>,
    #[serde(rename = "East Riverfront Station")]
    east_riverfront: Option<String>,
    #[serde(rename = "5th & Missouri Station")]
    fifth_missouri: Option<String>,
    #[serde(rename = "Emerson Park Station")]
    emerson_park: Option<String>,
    #[serde(rename = "JJK Center Station")]
    jjk: Option<String>,
    #[serde(rename = "Washington Park Station")]
    washington: Option<String>,
    #[serde(rename = "Fairview Heights Station")]
    fairview_heights: Option<String>,
    #[serde(rename = "Memorial Hospital Station")]
    memorial_hospital: Option<String>,
    #[serde(rename = "Swansea Station")]
    swansea: Option<String>,
    #[serde(rename = "Belleville Station")]
    belleville: Option<String>,
    #[serde(rename = "College Station")]
    college: Option<String>,
    #[serde(rename = "Shiloh-Scott Station")]
    shiloh_scott: Option<String>,
}

fn main() {
    server::new(|| {
        App::new().resource("/next-arrival", |r| r.method(http::Method::POST)
            .with(next_arrival))
    }).bind("0.0.0.0:8000").expect("Address already in use").run();
    println!("app started on port 8000");
}

fn parse_request_pick_file(t: DateTime<Local>, direction: &str)-> Option<String>{
    let day: &str = match t.weekday(){
        Weekday::Sat | Weekday::Sun => "weekend",
        _ => "weekday",
    };
    match direction{
        "east" | "west" => {
            println!("the file to open is {}bound-{}-schedule.csv", direction, day);
            return Some(format!("{}bound-{}-schedule.csv", direction, day))
        },
        _ => {
            println!("not east or west?");
            return None
        }
    };
}

fn next_arrival(req: Json<NextArrivalRequest>) -> HttpResponse {
    let input = req.into_inner();
    let t = Local::now();
    match parse_request_pick_file(t, input.direction.as_str()){
        Some(data) => {
            println!("{}", data.clone());
            match search_csv(data, input.station.clone()){
                Ok(s) => {
                    match serde_json::to_string(&NextArrivalResponse{
                        station: input.station,
                        direction: input.direction,
                        time: s
                    }) {
                        Ok(s) => return HttpResponse::Ok().content_type("application/json").body(s),
                        _ => return HttpResponse::BadRequest().reason("error building response").finish(),
                    }

                },
                _ => return HttpResponse::BadRequest().reason("error during match").finish()
            }
        },
        None => return HttpResponse::BadRequest().reason("direction must be 'east' or 'west'").finish()
    }
}

fn search_csv(filename: String, station: String) -> Result<String, Box<Error>>{
    let t = Local::now();
    match Asset::get(&filename) {
        Some(file_contents) => {
            let mut reader = csv::Reader::from_reader(&file_contents[..]);
            for result in reader.deserialize(){
                let record: StationTimeSlice = result?;
                println!("{:?}", record);
                match record.cwe { //hardcoding cwe for now
                    Some(s) => {
                        if schedule_time_is_later_than_now(s.clone()){
                            return Ok(s)
                        }
                    },
                    None => {
                        println!("continuing on none");
                        continue
                    } //empty field in csv; keep looking
                }
            }
            return Err(From::from("failed to find a time from schedule data"));
        },
        None => Err(From::from("failed to get embedded csv file"))
    }
}

fn schedule_time_is_later_than_now(s: String) -> bool{
    //s looks like 4:15A
    let mut st = s.clone();
    let mut plus_twelve = false;
    if st.pop().unwrap().to_string().eq("P"){
        plus_twelve = true;
    }
    println!("{}", st);
    let x: Vec<&str> = st.split(":").collect();
    let mut hh: u32 = x[0].parse::<u32>().unwrap();
    let mm: u32 = x[1].parse::<u32>().unwrap();
    if plus_twelve{
        hh = hh + 12;
    }
    let t = Local::now();
    if t.minute() < mm && t.hour() < hh{
        return true;
    }
    return false;
}
