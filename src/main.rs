#[macro_use]
extern crate rust_embed;
#[macro_use]
extern crate serde_derive;

use actix_web::{post, web, App, HttpResponse, HttpServer};
use chrono::{DateTime, Datelike, Local, Weekday};
use clap::{App as ClApp, Arg};
use std::cmp::Ordering;
use csv::Reader;

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
    line: String,
    time: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct StationTimeSlice {
    #[serde(rename = "Lambert Airport Terminal # 1")]
    lambert_t1: Option<String>,
    #[serde(rename = "Lambert Airport Terminal # 2")]
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
    #[serde(rename = "ShrewsburyLansdowne I44 Station")]
    shrewsbury: Option<String>,
    #[serde(rename = "Sunnen Station")]
    sunnen: Option<String>,
    #[serde(rename = "MaplewoodManchester Station")]
    maplewood_manchester: Option<String>,
    #[serde(rename = "Brentwood I64 Station")]
    brentwood: Option<String>,
    #[serde(rename = "Richmond Heights Station")]
    richmond_heights: Option<String>,
    #[serde(rename = "Clayton Station")]
    clayton: Option<String>,
    #[serde(rename = "Forsyth Station")]
    forsyth: Option<String>,
    #[serde(rename = "University CityBig Bend Station")]
    u_city: Option<String>,
    #[serde(rename = "Skinker Station")]
    skinker: Option<String>,
    #[serde(rename = "Forest ParkDeBaliviere Station")]
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
    #[serde(rename = "Laclede's Landing Station")]
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
    #[serde(rename = "ShilohScott Station")]
    shiloh_scott: Option<String>,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args = ClApp::new("metro-schedule-api")
        .arg(Arg::with_name("port").help("port number for webserver"))
        .get_matches();
    let port = args.value_of("port").unwrap_or("8000");
    println!("app starting on port {}", &port);
    HttpServer::new(|| App::new().service(next_arrival))
        .bind(format!("0.0.0.0:{}", port))?
        .run()
        .await
}

#[post("/next-arrival")]
fn next_arrival(req: web::Json<NextArrivalRequest>) -> HttpResponse {
    let input = req.into_inner();
    let t = Local::now();
    match parse_request_pick_file(t, input.direction.as_str()) {
        Some(data) => match Asset::get(&data) {
            Some(file_contents) => {
                match search_csv(&file_contents, input.station.to_lowercase().as_str(), t) {
                    Ok(s) => match serde_json::to_string(&NextArrivalResponse {
                        station: input.station,
                        direction: input.direction,
                        line: s.1,
                        time: s.0,
                    }) {
                        Ok(s) => HttpResponse::Ok().content_type("application/json").body(s),
                        Err(_) => HttpResponse::InternalServerError().into(),
                    },
                    Err(_) => HttpResponse::InternalServerError().into(),
                }
            }
            None => HttpResponse::InternalServerError().into(),
        },
        None => HttpResponse::BadRequest()
            .reason("direction must be 'east' or 'west'")
            .finish(),
    }
}

fn parse_request_pick_file(t: DateTime<Local>, direction: &str) -> Option<String> {
    let day: &str = match t.weekday() {
        Weekday::Sat => "saturday",
        Weekday::Sun => "sunday",
        _ => "weekday",
    };
    match direction {
        "east" | "west" => Some(format!("{}bound-{}-schedule.csv", direction, day)),
        _ => {
            println!("not east or west?");
            None
        }
    }
}

fn search_csv(
    file_contents: &[u8],
    station: &str,
    t: DateTime<Local>,
) -> Result<(String, String), &'static str> {
    let mut reader = Reader::from_reader(&file_contents[..]);
    match station {
        "lambert" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.lambert_t1 {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "lambert2" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.lambert_t2 {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "hanley" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.north_hanley {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "umsl north" | "umsl" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.umsl_north {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "umsl south" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.umsl_south {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "rock road" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.rock_road {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "wellston" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.wellston {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "delmar" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.delmar_loop {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "shrewsbury" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.shrewsbury {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "sunnen" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.sunnen {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "maplewood" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.maplewood_manchester {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "brentwood" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.brentwood {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "richmond" | "richmond heights" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.richmond_heights {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "clayton" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.clayton {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "forsyth" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.forsyth {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "u city" | "university city" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.u_city {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "skinker" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.skinker {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "forest park" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.forest_park {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "cwe" | "central west end" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.cwe {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "cortex" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.cortex {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "grand" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.grand {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "union" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.union {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "civic center" | "civic" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.civic_center {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "stadium" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.stadium {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "8th and pine" | "8th pine" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.eight_pine {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "convention center" | "convention" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.convention_center {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "lacledes" | "lacledes landing" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.lacledes_landing {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "riverfront" | "east riverfront" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.east_riverfront {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "fifth missouri" | "5th missouri" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.fifth_missouri {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "emerson" | "emerson park" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.emerson_park {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "jjk" | "jackie joiner" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.jjk {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "washington" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.washington {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "fvh" | "fairview heights" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.fairview_heights {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "memorial hospital" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.memorial_hospital {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "swansea" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.swansea {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "belleville" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.belleville {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "college" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.college {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        "shiloh" | "shiloh scott" => {
            for result in reader.deserialize() {
                let record: StationTimeSlice = result.unwrap();
                match record.shiloh_scott {
                    Some(s) => {
                        if schedule_time_is_later_than_now(t, s.clone()) {
                            return Ok(line_info(s));
                        }
                    }
                    None => continue,
                }
            }
            Err("failed to find a time from schedule data")
        }
        _ => Err("that station is not in the schedule"),
    }
}

fn schedule_time_is_later_than_now(t: DateTime<Local>, mut s: String) -> bool {
    let mut plus_twelve = false;
    let _ = s.pop(); //remove line type
    if s.pop().unwrap().to_string().eq("P") {
        plus_twelve = true;
    }
    let x: Vec<&str> = s.split(':').collect();
    let mut hh: u32 = x[0].parse::<u32>().unwrap_or_default();
    let mm: u32 = x[1].parse::<u32>().unwrap_or_default();
    if plus_twelve {
        hh = ((hh % 12) + 12) % 24;
    }
    match t.cmp(&Local::today().and_hms(hh, mm, 00)) {
        Ordering::Less => true,
        Ordering::Equal => true,
        Ordering::Greater => false,
    }
}

fn line_info(mut s: String) -> (String, String) {
    let line = match s.pop() {
        Some(c) => match c {
            'R' => "red",
            'B' => "blue",
            _ => "",
        },
        None => "",
    };
    (s, line.to_string())
}
