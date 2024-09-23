use chrono::prelude::*;
use chrono::NaiveDate;
use moka::future::Cache;
use rocket::http::Method;
use rocket::http::{ContentType, Header};
use rocket::response::{Responder, Response};
use rocket::Request;
use rocket::State;
use rocket_cors::{AllowedOrigins, CorsOptions};
use std::io::Cursor;
use std::time::Duration;

mod chmi_api;

pub use crate::chmi_api::get_all;
pub use crate::chmi_api::get_image;

#[macro_use]
extern crate rocket;

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
            cache: Cache::builder()
                .time_to_live(Duration::from_secs(past as u64 * 5 * 60))
                .build(),
        }
    }
}

#[get("/api/get")]
async fn get_list(state: &State<GlobalState>) -> CustomBinaryResponse {
    // Downloading image and returning it. Cache can be cloned, but works globally, so no worries.
    let arr = get_all(state.past, &state.url, state.cache.clone()).await;
    // Returning Vector was bit problematic, so creating string my self.
    let json = serde_json::to_string(&arr).unwrap();
    // To get ContentType, I had to use Custom struct to return. And because we already have binary
    // one, so I used it. But string has got to be converted to Vec<u8>.
    CustomBinaryResponse {
        data: json.as_bytes().to_vec(),
        content_type: ContentType::JSON,
        custom_headers: vec![],
    }
}

#[get("/api/get/<year>/<month>/<day>/<hour>/<minute>")]
async fn get_single(
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    state: &State<GlobalState>,
) -> Option<CustomBinaryResponse> {
    // Validate date to be valid. 31 days in every month, but it's good enough. Next validation
    // will catch those, so it SHOULDN'T be that bad.
    if year < 2000
        || month < 1
        || month > 12
        || day < 1
        || day > 31
        || hour > 23
        || minute > 59
        || minute % 5 != 0
    {
        return None;
    }

    // Converting to dates and than to minutes after 1970 (we don't need milliseconds or seconds in
    // this app.
    let requested_date = NaiveDate::from_ymd_opt(year.into(), month.into(), day.into())?
        .and_hms_opt(hour.into(), minute.into(), 0)?
        .and_utc();
    let requested_time = requested_date.timestamp_millis() / 1000 / 60;
    let current_date = Utc::now().to_utc();
    let current_time = current_date.timestamp_millis() / 1000 / 60;

    // Check if time is in range of available to fetch. Magic constant 5 is there, because images
    // from chmi.cz are 5 minutes apart. And magic 1 is there, because it can take few seconds to
    // generate image, so just in case allow to fetch bit older image.
    if requested_time > current_time || requested_time < current_time - (state.past * 5) - 1 {
        return None;
    }

    // Downloading image and returning it. Cache can be cloned, but works globally, so no worries.
    let image = get_image(year, month, day, hour, minute, state.cache.clone()).await;
    match image {
        Some(data) => Some(CustomBinaryResponse {
            data,
            content_type: ContentType::Binary,
            custom_headers: vec![
                Header::new("Content-Type", "image/png"),
                Header::new(
                    "Content-Disposition",
                    format!(
                        "inline; filename=meteo-{}-{}-{}-{}-{}.png",
                        year, month, day, hour, minute
                    ),
                ),
            ],
        }),
        None => None,
    }
}

#[launch]
fn rocket() -> _ {
    // Yes, I should restrict to single site, but...
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(vec![Method::Get].into_iter().map(From::from).collect())
        .allow_credentials(true);
    rocket::build()
        .manage(GlobalState::new())
        .attach(cors.to_cors().unwrap())
        .mount("/", routes![get_list])
        .mount("/", routes![get_single])
}
