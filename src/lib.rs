use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use units::Units;
use weather::Weather;

pub mod persistence;
pub mod units;
pub mod weather;

mod date_format;
mod emoji;
mod geo;
mod wind;

pub const DATE_INPUT_FORMAT: &str = "%Y-%m-%d %H:%M";

fn date_parser(value: &String) -> Result<NaiveDateTime> {
    NaiveDateTime::parse_from_str(value, DATE_INPUT_FORMAT).with_context(|| {
        format!("Failed to parse date and time, it should be in the format {DATE_INPUT_FORMAT}: {value}")
    })
}

#[derive(Debug, PartialEq)]
pub struct Event {
    pub name: Option<String>,
    pub when: NaiveDateTime,
    location: String,
    latitude: f64,
    longitude: f64,
    days: i64,
}

impl Event {
    pub async fn new(name: Option<String>, date: String, location: String) -> Result<Self> {
        let when = date_parser(&date)?;
        let (latitude, longitude) = geo::coordinates(&location).await?;

        Ok(Self {
            name,
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
                match &self.name {
                    Some(name) => println!(
                        "Skipping weather forecast for {} since it was {} days ago.",
                        name, -self.days
                    ),
                    None => println!("Skipping weather forecast for {} days ago.", -self.days),
                };
            }
            return false;
        }
        if self.days >= 16 {
            if verbose {
                match &self.name {
                    Some(name) => println!(
                        "Skipping weather forecast for {} since it is {} days in the future.",
                        name, self.days
                    ),

                    None => println!(
                        "Skipping weather forecast for {} days in the future.",
                        self.days
                    ),
                };
            }
            return false;
        }
        true
    }

    pub async fn weather(&self, units: &Units) -> Result<Weather> {
        Weather::new(
            self.when,
            self.latitude,
            self.longitude,
            units,
            self.name.clone(),
            self.location.clone(),
        )
        .await
    }
}
