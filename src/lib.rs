use anyhow::Result;

use chrono::NaiveDateTime;

mod emoji;
mod forecast;
mod geo;
mod open_weather_date_format;
pub mod persistence;
mod wind;

pub struct Event {
    pub name: Option<String>,
    when: NaiveDateTime,
    location: String,
    latitude: f64,
    longitude: f64,
    days: i64,
}

impl Event {
    pub async fn new(name: Option<String>, when: NaiveDateTime, location: String) -> Result<Self> {
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
        if self.days > 5 {
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

    pub async fn weather(&self, json: bool) -> Result<String> {
        let weather = forecast::Forecast::new(self.latitude, self.longitude)?
            .five_days(self.name.clone(), self.location.clone(), self.when)
            .await?;

        if json {
            weather.as_json()
        } else {
            weather.as_string()
        }
    }
}
