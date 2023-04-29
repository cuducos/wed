use std::env;

use anyhow::{Context, Result};
use reqwest::Url;

const MISSING_API_KEY_ERROR: &str = "Couldn't find the OpenWeather API key as an
environment variable called OPEN_WEATHER_API_KEY. You need to create
one. It's free.
* Create an account at https://home.openweathermap.org/users
* Get the key from https://home.openweathermap.org/api_keys";

pub struct Forecast {
    api_key: String,
    latitude: f64,
    longitude: f64,
}

impl Forecast {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self> {
        Ok(Self {
            latitude,
            longitude,
            api_key: env::var("OPEN_WEATHER_API_KEY").with_context(|| MISSING_API_KEY_ERROR)?,
        })
    }

    pub fn four_days(&self) -> Result<()> {
        self.todo(4, "pro", "hourly")
    }

    pub fn sixteen_days(&self) -> Result<()> {
        self.todo(16, "api", "daily")
    }

    pub fn thirty_days(&self) -> Result<()> {
        self.todo(30, "pro", "climate")
    }

    fn url(&self, subdomain: &str, endpoint: &str) -> Result<Url> {
        let base_url =
            format!("https://{subdomain}.openweathermap.org/data/2.5/forecast/{endpoint}");
        let url = Url::parse_with_params(
            base_url.as_str(),
            &[
                ("lat", self.latitude.to_string()),
                ("lon", self.longitude.to_string()),
                ("appid", self.api_key.clone()),
                ("unit", "metric".to_string()), // TODO: CLI option for unit
                ("lang", "en".to_string()),     // TODO: CLI option for language
            ],
        )?;

        Ok(url)
    }

    fn todo(&self, days: i64, subdomain: &str, endpoint: &str) -> Result<()> {
        let url = self.url(subdomain, endpoint)?;
        println!("TODO: {days} days forecast {url}");
        Ok(())
    }
}
