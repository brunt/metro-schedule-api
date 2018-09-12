#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate serde_derive;
extern crate actix_web;
extern crate chrono;
extern crate csv;
extern crate serde;
extern crate serde_json;

use actix_web::{http, server, App, HttpResponse, Json};
use chrono::{DateTime, Datelike, Local, Weekday};
use std::cmp::Ordering;
use std::error::Error;

#[derive(RustEmbed)]
#[folder = "data/"]
struct Asset;

#[derive(Serialize, Deserialize)]
struct NextArrivalRequest {
    station: String,
    direction: String,
}

#[derive(Serialize)]
struct NextArrivalResponse {
    station: String,
    direction: String,
    time: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct StationTimeSlice {
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
    println!("app started on port 8000");
    server::new(|| {
        App::new().resource("/next-arrival", |r| {
            r.method(http::Method::POST).with(next_arrival)
        })
    }).bind("0.0.0.0:8000")
        .expect("Address already in use")
        .run();
}

fn next_arrival(req: Json<NextArrivalRequest>) -> HttpResponse {
    let input = req.into_inner();
    let t = Local::now();
    match parse_request_pick_file(t, input.direction.as_str()) {
        Some(data) => match search_csv(data, input.station.to_lowercase().as_str(), t) {
            Ok(s) => match serde_json::to_string(&NextArrivalResponse {
                station: input.station,
                direction: input.direction,
                time: s,
            }) {
                Ok(s) => return HttpResponse::Ok().content_type("application/json").body(s),
                _ => {
                    return HttpResponse::BadRequest()
                        .reason("error building response")
                        .finish()
                }
            },
            Err(e) => {
                println!("{:?}", e.description());
                return HttpResponse::BadRequest()
                    .reason("error during match")
                    .finish();
            }
        },
        None => {
            return HttpResponse::BadRequest()
                .reason("direction must be 'east' or 'west'")
                .finish()
        }
    }
}

fn parse_request_pick_file(t: DateTime<Local>, direction: &str) -> Option<String> {
    let day: &str = match t.weekday() {
        Weekday::Sat | Weekday::Sun => "weekend",
        _ => "weekday",
    };
    match direction {
        "east" | "west" => return Some(format!("{}bound-{}-schedule.csv", direction, day)),
        _ => {
            println!("not east or west?");
            return None;
        }
    };
}

fn search_csv(filename: String, station: &str, t: DateTime<Local>) -> Result<String, Box<Error>> {
    match Asset::get(&filename) {
        Some(file_contents) => {
            let mut reader = csv::Reader::from_reader(&file_contents[..]);
            for result in reader.deserialize() {
                let record: StationTimeSlice = result?;
                match station {
                    "lambert" => {
                        match record.lambert_t1 {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "lambert2" => {
                        match record.lambert_t2 {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "hanley" => {
                        match record.north_hanley {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "umsl north" | "umsl" => {
                        match record.umsl_north {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "umsl south" => {
                        match record.umsl_south {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "rock road" => {
                        match record.rock_road {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    }
                    "wellston" => {
                        match record.wellston {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "delmar" => {
                        match record.delmar_loop {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "shrewsbury" => {
                        match record.shrewsbury {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "sunnen" => {
                        match record.sunnen {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "maplewood" => {
                        match record.maplewood_manchester {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "brentwood" => {
                        match record.brentwood {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "richmond" | "richmond heights" => {
                        match record.richmond_heights {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "clayton" => {
                        match record.clayton {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "forsyth" => {
                        match record.forsyth {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "u city" | "university city" => {
                        match record.u_city {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "skinker" => {
                        match record.skinker {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "forest park" => {
                        match record.forest_park {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "cwe" | "central west end" => {
                        match record.cwe {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "cortex" => {
                        match record.cortex {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "grand" => {
                        match record.grand {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "union" => {
                        match record.union {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "civic center" | "civic" => {
                        match record.civic_center {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "stadium" => {
                        match record.stadium {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "8th and pine" | "8th pine" => {
                        match record.eight_pine {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "convention center" | "convention" => {
                        match record.convention_center {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "lacledes" | "lacledes landing" => {
                        match record.lacledes_landing {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "east riverfront" => {
                        match record.east_riverfront {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "fifth missouri" | "5th missouri" => {
                        match record.fifth_missouri {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "emerson" | "emerson park" => {
                        match record.emerson_park {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "jjk" | "jackie joiner" => {
                        match record.jjk {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "washington" => {
                        match record.washington {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "fvh" | "fairview heights" => {
                        match record.fairview_heights {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "memorial hospital" => {
                        match record.memorial_hospital {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "swansea" => {
                        match record.swansea {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "belleville" => {
                        match record.belleville {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "college" => {
                        match record.college {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    "shiloh" | "shiloh scott" => {
                        match record.shiloh_scott {
                            Some(s) => if schedule_time_is_later_than_now(t, s.clone()){
                                return Ok(s)
                            }
                            None => continue,
                        }
                    },
                    _ => return Err(From::from("that station is not in the schedule")),
                }
            }
            return Err(From::from("failed to find a time from schedule data"));
        }
        None => Err(From::from("failed to get embedded csv file")),
    }
}

fn schedule_time_is_later_than_now(t: DateTime<Local>, mut s: String) -> bool {
    let mut plus_twelve = false;
    if s.pop().unwrap().to_string().eq("P") {
        plus_twelve = true;
    }
    let x: Vec<&str> = s.split(":").collect();
    let mut hh: u32 = x[0].parse::<u32>().unwrap_or_default();
    let mm: u32 = x[1].parse::<u32>().unwrap_or_default();
    if plus_twelve {
        hh = ((hh % 12) + 12) % 24;
    }
    match t.cmp(&Local::today().and_hms(hh, mm, 00)) {
        Ordering::Less => return true,
        Ordering::Equal => return true,
        Ordering::Greater => return false,
    }
}
