use std::env;

use anyhow::{anyhow, Context, Result};
use chrono::NaiveDateTime;
use reqwest::{Client, Url};
use serde::{self, Deserialize};

const MISSING_API_KEY_ERROR: &str = "Couldn't find the OpenWeather API key as an
environment variable called OPEN_WEATHER_API_KEY. You need to create
one. It's free.
* Create an account at https://home.openweathermap.org/users
* Get the key from https://home.openweathermap.org/api_keys";

mod my_date_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").map_err(serde::de::Error::custom)
    }
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
struct Weather {
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
    #[serde(with = "my_date_format")]
    dt_txt: NaiveDateTime,
    main: Main,
    pop: f64,
    weather: Vec<Weather>,
    wind: Wind,
}

const API_BASE_URL: &str = "https://api.openweathermap.org/data/2.5/forecast";

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

    pub async fn five_days(&self) -> Result<()> {
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
        for item in data.list {
            println!("{}", item.dt_txt);
            println!("{}", item.pop);
            println!("{}", item.main.temp);
            println!("{}", item.main.feels_like);
            println!("{}", item.main.humidity);
            println!("{}", item.wind.speed);
            println!("{}", item.wind.deg);
            for weather in item.weather {
                println!("{}", weather.main);
                println!("{}", weather.description);
            }
        }
        Ok(())
    }
}
