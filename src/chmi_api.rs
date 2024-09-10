use serde::Serialize;
use surf::{get, StatusCode};
use image::{DynamicImage, ImageOutputFormat};
use std::io::Cursor;
use image::imageops::crop_imm;
use chrono::prelude::*;
use chrono::DateTime;
use moka::future::Cache;

#[derive(Serialize)]
pub struct LabelUrl {
    label: String,
    url: String,
}

fn f (input: u32) -> String {
    if input <= 9 {
        return format!("0{}", input);
    }
    format!("{}", input)
}

pub async fn get_all(past: i64, url: &String, cache: Cache<String, Vec<u8>>) -> Vec<LabelUrl> {
    let current_date = Utc::now().to_utc();
    let current_time = ((current_date.timestamp_millis() as f64 / 1000 as f64/ 60 as f64/ 5 as f64).floor() as i64) * 5 * 60 * 1000;
    let mut last_possible_date = DateTime::from_timestamp_millis(current_time.into()).unwrap();
    let possible_image = get_image(
        last_possible_date.year().try_into().unwrap(),
        last_possible_date.month().try_into().unwrap(),
        last_possible_date.day().try_into().unwrap(),
        last_possible_date.hour().try_into().unwrap(),
        last_possible_date.minute().try_into().unwrap(),
        cache,
    ).await;
    if possible_image == None {
        last_possible_date = DateTime::from_timestamp_millis(last_possible_date.timestamp_millis() - 5 * 60 * 1000).unwrap();
    }

    let mut arr: Vec<LabelUrl> = Vec::new();
    for i in 0..past {
        let date = DateTime::from_timestamp_millis(last_possible_date.timestamp_millis() - i * 5 * 60 * 1000).unwrap();
        let local_date: DateTime<Local> = DateTime::from(date);
        arr.push(LabelUrl {
            label: format!("{}:{}", local_date.hour(), f(local_date.minute())),
            url: format!("{}/api/get/{}/{}/{}/{}/{}", url, date.year(), date.month(), date.day(), date.hour(), date.minute()),
        });
    }
    arr
}

pub async fn get_image(year: u16, month: u8, day: u8, hour: u8, minute: u8, cache: Cache<String, Vec<u8>>) -> Option<Vec<u8>> {
    let url = format!("https://www.chmi.cz/files/portal/docs/meteo/rad/inca-cz/data/czrad-z_max3d/pacz2gmaps3.z_max3d.{}{}{}.{}{}.0.png", year, f(month.into()), f(day.into()), f(hour.into()), f(minute.into()));
    let cached = cache.get(&url).await;
    if cached != None {
        return Some(cached.unwrap());
    }

    let mut response = get(url.clone()).await.expect("Failed to download image");
    if response.status() != StatusCode::Ok {
        return None;
    }
    let data = response.body_bytes().await.expect("Failed to get body");

    let img = image::load_from_memory(data.as_slice()).unwrap();
    let img_resized = crop_imm(&img, 1, 95, 597, 320);
    let img_resized_converted = DynamicImage::ImageRgba8(img_resized.to_image());
    
    let mut buf = Vec::new();
    img_resized_converted.write_to(&mut Cursor::new(&mut buf), ImageOutputFormat::Png)
        .expect("Failed to write image to buffer");
    cache.insert(url, buf.clone()).await;

    Some(buf)
}
