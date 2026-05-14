use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use reqwest::{Client, Response, Url};
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

#[derive(Clone)]
pub struct WedClient {
    client: Client,
    retries: u64,
}

impl WedClient {
    pub fn new(retries: u64) -> Result<Self> {
        let user_agent = format!(
            "{}/{} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("CARGO_PKG_REPOSITORY"),
        );
        let client = Client::builder().user_agent(user_agent).build()?;
        Ok(Self { client, retries })
    }

    pub async fn get(&self, url: Url) -> Result<Response> {
        let mut retries = 0;
        loop {
            let resp = self.client.get(url.clone()).send().await?;
            if resp.status() == 429 && retries < self.retries {
                retries += 1;
                tokio::time::sleep(std::time::Duration::from_secs(retries)).await;
                continue;
            }
            break Ok(resp);
        }
    }
}

fn date_parser(value: &str) -> Result<NaiveDateTime> {
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
}

impl Event {
    pub async fn new(
        client: &WedClient,
        name: Option<String>,
        date: String,
        location: String,
    ) -> Result<Self> {
        let when = date_parser(&date)?;
        let (latitude, longitude) = geo::coordinates(client, &location).await?;

        Ok(Self {
            name,
            when,
            location,
            latitude,
            longitude,
        })
    }

    fn countdown_in_days(&self) -> i64 {
        (self.when - chrono::Local::now().naive_local()).num_days()
    }

    pub fn has_weather_forecast(&self, verbose: bool) -> bool {
        let days = self.countdown_in_days();
        if days < 0 {
            if verbose {
                match &self.name {
                    Some(name) => println!(
                        "Skipping weather forecast for {} since it was {} days ago.",
                        name, -days
                    ),
                    None => println!("Skipping weather forecast for {} days ago.", -days),
                };
            }
            return false;
        }
        if days >= 16 {
            if verbose {
                match &self.name {
                    Some(name) => println!(
                        "Skipping weather forecast for {} since it is {} days in the future.",
                        name, days
                    ),

                    None => println!("Skipping weather forecast for {} days in the future.", days),
                };
            }
            return false;
        }
        true
    }

    pub async fn weather(&self, client: &WedClient, units: &Units) -> Result<Weather<'_>> {
        Weather::new(
            client,
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
