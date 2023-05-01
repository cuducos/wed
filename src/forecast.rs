use std::env;

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;
use reqwest::{Client, Url};
use serde::{self, Deserialize, Serialize};

use crate::open_weather_date_format;

const API_BASE_URL: &str = "https://api.openweathermap.org/data/2.5/forecast";
const MISSING_API_KEY_ERROR: &str = "Couldn't find the OpenWeather API key as an
environment variable called OPEN_WEATHER_API_KEY. You need to create
one. It's free.
* Create an account at https://home.openweathermap.org/users
* Get the key from https://home.openweathermap.org/api_keys";

#[derive(Serialize, Debug)]
pub struct Weather {
    #[serde(with = "open_weather_date_format")]
    pub date: NaiveDateTime,
    pub title: String,
    pub description: String,
    pub probability_of_precipitation: f64,
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: f64,
    pub wind_speed: f64,
    pub wind_direction: f64, // TODO: convert to N/E/S/W
}

#[derive(Deserialize, Debug)]
struct Response {
    list: Vec<ResponseListItem>,
}

#[derive(Deserialize, Debug)]
struct Main {
    temp: f64,
    feels_like: f64,
    humidity: f64,
}

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    main: String,
    description: String,
}

#[derive(Deserialize, Debug)]
struct Wind {
    speed: f64,
    deg: f64,
}

#[derive(Deserialize, Debug)]
struct ResponseListItem {
    #[serde(with = "open_weather_date_format")]
    dt_txt: NaiveDateTime,
    main: Main,
    pop: f64,
    weather: Vec<WeatherResponse>,
    wind: Wind,
}

impl ResponseListItem {
    fn as_weather(&self) -> Result<Weather> {
        match self.weather.first() {
            None => Err(anyhow!("No weather response found")),
            Some(weather) => Ok(Weather {
                date: self.dt_txt,
                title: weather.main.clone(),
                description: weather.description.clone(),
                probability_of_precipitation: self.pop,
                temperature: self.main.temp,
                feels_like: self.main.feels_like,
                humidity: self.main.humidity,
                wind_speed: self.wind.speed,
                wind_direction: self.wind.deg,
            }),
        }
    }
}

pub struct Forecast {
    url: Url,
}

impl Forecast {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self> {
        let api_key = env::var("OPEN_WEATHER_API_KEY").with_context(|| MISSING_API_KEY_ERROR)?;

        Ok(Self {
            url: Url::parse_with_params(
                API_BASE_URL,
                &[
                    ("appid", api_key),
                    ("lat", latitude.to_string()),
                    ("lon", longitude.to_string()),
                    ("cnt", "40".to_string()),
                    ("units", "metric".to_string()), // TODO: CLI option for unit
                    ("lang", "en".to_string()),      // TODO: CLI option for language
                ],
            )?,
        })
    }

    pub async fn five_days(&self, target: NaiveDateTime) -> Result<Weather> {
        let resp = Client::new().get(self.url.to_string()).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!(
                "HTTP request to {} returned {}: {}",
                &self.url,
                resp.status(),
                resp.text().await?
            ));
        }

        let data: Response = resp.json().await?;
        let item = data
            .list
            .iter()
            .min_by_key(|a| a.dt_txt.signed_duration_since(target).num_seconds().abs());

        item.ok_or(anyhow!("No weather data found"))
            .and_then(|item| item.as_weather())
    }
}
