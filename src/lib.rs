use anyhow::Result;
use std::fmt;

use chrono::NaiveDateTime;

mod forecast;
mod geo;

const DATE_OUTPUT_FORMAT: &str = "%b %-d, %H:%M";

pub struct Event {
    when: NaiveDateTime,
    location: String,
    latitude: f64,
    longitude: f64,
    days: i64,
}

impl Event {
    pub async fn new(when: NaiveDateTime, location: String) -> Result<Self> {
        let (latitude, longitude) = geo::coordinates(&location).await?;

        Ok(Self {
            when,
            location,
            latitude,
            longitude,
            days: (when - chrono::Local::now().naive_local()).num_days(),
        })
    }

    pub fn has_weather_forcast(&self, verbose: bool) -> bool {
        if self.days < 0 {
            if verbose {
                println!("Skipping weather forecast for {} days ago.", -self.days);
            }
            return false;
        }
        if self.days > 5 {
            if verbose {
                println!(
                    "Skipping weather forecast for {} days in the future.",
                    self.days
                );
            }
            return false;
        }
        return true;
    }

    pub fn weather(&self) -> Result<()> {
        if !self.has_weather_forcast(true) {
            return Ok(());
        }

        forecast::Forecast::new(self.latitude, self.longitude)?.five_days()
    }
}

impl fmt::Display for Event {
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
