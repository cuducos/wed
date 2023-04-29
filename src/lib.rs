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

    pub fn is_in_the_past(&self) -> bool {
        self.days < 0
    }

    pub fn weather(&self) -> Result<()> {
        let weather = forecast::Forecast::new(self.latitude, self.longitude)?;

            match self.days {
            0..=4 => weather.four_days(),
            5..=16 => weather.sixteen_days(),
            17..=30 => weather.thirty_days(),
            _ => (),
        };

    Ok(())
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
