use anyhow::Result;
use std::fmt;

use chrono::NaiveDateTime;

mod nominatim;
mod open_weather_map;

const DATE_OUTPUT_FORMAT: &str = "%b %-d, %H:%M";

pub struct Forecast {
    when: NaiveDateTime,
    location: String,
    latitude: f64,
    longitude: f64,
    days: i64,
}

impl Forecast {
    pub async fn new(when: NaiveDateTime, location: String) -> Result<Self> {
        let (latitude, longitude) = nominatim::geo_location_for(&location).await?;

        Ok(Self {
            when,
            location,
            latitude,
            longitude,
            days: (when - chrono::Local::now().naive_local()).num_days(),
        })
    }

    pub fn is_in_the_past(&self) -> bool {
        self.days < 0
    }

    pub fn api(&self) {
        match self.days {
            0..=4 => open_weather_map::four_days_forecast(self.latitude, self.longitude),
            5..=16 => open_weather_map::sixteen_days_forecast(self.latitude, self.longitude),
            17..=30 => open_weather_map::thirty_days_forecast(self.latitude, self.longitude),
            _ => (),
        };
    }
}

impl fmt::Display for Forecast {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Where\t{} ({}, {})\nDate\t{}",
            self.location,
            self.latitude,
            self.longitude,
            self.when.format(DATE_OUTPUT_FORMAT),
        )
    }
}
