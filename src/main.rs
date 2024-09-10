use chrono::NaiveDate;
use rocket::http::{ContentType, Header};
use rocket::response::{Responder, Response};
use rocket::Request;
use std::io::Cursor;
use chrono::prelude::*;
use rocket::State;
use moka::future::Cache;

mod chmi_api;

pub use crate::chmi_api::get_image;
pub use crate::chmi_api::get_all;

#[macro_use] extern crate rocket;

// Generated with ChatGPT
pub struct CustomBinaryResponse {
    data: Vec<u8>,
    content_type: ContentType,
    custom_headers: Vec<Header<'static>>,
}
impl<'r> Responder<'r, 'static> for CustomBinaryResponse {
    fn respond_to(self, _request: &'r Request<'_>) -> rocket::response::Result<'static> {
        let mut response = Response::build();

        // Set the content type
        response.header(self.content_type);

        // Add custom headers
        for header in self.custom_headers {
            response.header(header);
        }

        // Set the body as a cursor of the binary data
        response.sized_body(self.data.len(), Cursor::new(self.data));

        response.ok()
    }
}
// End of generated with ChatGPT

struct GlobalState {
    pub past: i64,
    pub url: String,
    pub cache: Cache<String, Vec<u8>>,
}

impl GlobalState {
    pub fn new() -> Self {
        let past = match std::env::var("PAST") {
            Ok(x) => x.parse::<i64>().unwrap_or(24),
            _ => 24,
        };
        GlobalState {
            past,
            url: std::env::var("URL").unwrap_or(format!("http://localhost:8000")),
            cache: Cache::new(100),
        }
    }
}

#[get("/api/get")]
async fn get_list(state: &State<GlobalState>) -> CustomBinaryResponse {
    let arr = get_all(state.past, &state.url, state.cache.clone()).await;
    let json = serde_json::to_string(&arr).unwrap();
    CustomBinaryResponse {
        data: json.as_bytes().to_vec(),
        content_type: ContentType::JSON,
        custom_headers: vec![],
    }
}

#[get("/api/get/<year>/<month>/<day>/<hour>/<minute>")]
async fn get_single(year: u16, month: u8, day: u8, hour: u8, minute: u8, state: &State<GlobalState>) -> Option<CustomBinaryResponse> {
    if year < 2000 || month < 1 || month > 12  || day < 1 || day > 31 || hour > 23 || minute > 59 || minute % 5 != 0 {
        return None
    }

    let requested_date = NaiveDate::from_ymd_opt(year.into(), month.into(), day.into())?.and_hms_opt(hour.into(), minute.into(), 0)?.and_utc(); 
    let requested_time = requested_date.timestamp_millis() / 1000 / 60;
    let current_date = Utc::now().to_utc();
    let current_time = current_date.timestamp_millis() / 1000 / 60;

    if requested_time > current_time || requested_time < current_time - (state.past * 5) - 1 {
        return None;
    }

    let image = get_image(year, month, day, hour, minute, state.cache.clone()).await;
    match image {
        Some(data) => Some(CustomBinaryResponse {
            data,
            content_type: ContentType::Binary,
            custom_headers: vec![
                Header::new("Content-Type", "image/png"),
                Header::new("Content-Disposition", format!("attachment; filename=meteo-{}-{}-{}-{}-{}.png", year, month, day, hour, minute)),
            ],
        }),
        None => None
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(GlobalState::new())
        .mount("/", routes![get_list])
        .mount("/", routes![get_single])
}

