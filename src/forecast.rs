use std::env;

use anyhow::{Context, Result};
use reqwest::Url;

const MISSING_API_KEY_ERROR: &str = "Couldn't find the OpenWeather API key as an
environment variable called OPEN_WEATHER_API_KEY. You need to create
one. It's free.
* Create an account at https://home.openweathermap.org/users
* Get the key from https://home.openweathermap.org/api_keys";

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
                    ("unit", "metric".to_string()), // TODO: CLI option for unit
                    ("lang", "en".to_string()),     // TODO: CLI option for language
                ],
            )?,
        })
    }

    pub fn five_days(&self) -> Result<()> {
        println!("{}", self.url);
        Ok(())
    }
}
