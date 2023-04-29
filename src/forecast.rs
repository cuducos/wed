use std::env;

use anyhow::{Context, Result};

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

    pub fn four_days(&self) {
        self.todo(4)
    }

    pub fn sixteen_days(&self) {
        self.todo(16)
    }

    pub fn thirty_days(&self) {
        self.todo(30)
    }

    fn todo(&self, days: i64) {
        println!(
            "TODO: {} days forecast API for {}, {} with API key starting with {}…",
            days,
            self.latitude,
            self.longitude,
            self.api_key.get(..3).unwrap()
        );
    }
}
