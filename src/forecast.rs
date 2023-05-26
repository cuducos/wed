use anyhow::{anyhow, Result};
use chrono::{NaiveDateTime, ParseError};
use clap::ValueEnum;
use reqwest::{Client, Url};
use serde::{self, Deserialize, Serialize};

use crate::date_format::{self, OPEN_METEO_DATE_FORMAT};
use crate::emoji::{self, emoji_for_weather};
use crate::wind;

const API_URL: &str = "https://api.open-meteo.com/v1/forecast";
const DATE_OUTPUT_FORMAT: &str = "%b %-d, %H:%M";

#[derive(Debug, Clone, ValueEnum, Serialize)]
pub enum Units {
    Metric,
    Imperial,
}

impl Units {
    pub fn temperature(&self) -> String {
        match self {
            Units::Metric => "celsius",
            Units::Imperial => "fahrenheit",
        }
        .to_string()
    }
    pub fn speed(&self) -> String {
        match self {
            Units::Metric => "kmh",
            Units::Imperial => "mph",
        }
        .to_string()
    }
}

#[derive(Serialize, Debug)]
pub struct Weather {
    pub name: Option<String>,
    pub location: String,
    pub units: Units,
    pub icon: String,

    #[serde(with = "date_format")]
    pub date: NaiveDateTime,
    pub weather_code: i8,
    pub probability_of_precipitation: Option<i8>,
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: i8,
    pub wind_speed: f64,
    pub wind_direction: i32,
}

impl Weather {
    pub async fn new(
        when: NaiveDateTime,
        latitude: f64,
        longitude: f64,
        units: &Units,
        name: Option<String>,
        location: String,
    ) -> Result<Self> {
        let date = when.format("%Y-%m-%d");
        let params = [
            "temperature_2m",
            "apparent_temperature",
            "precipitation_probability",
            "relativehumidity_2m",
            "windspeed_10m",
            "winddirection_10m",
            "weathercode",
        ]
        .join(",");
        let url = Url::parse_with_params(
            API_URL,
            &[
                ("latitude", latitude.to_string()),
                ("longitude", longitude.to_string()),
                ("start_date", date.to_string()),
                ("end_date", date.to_string()),
                ("temperature_unit", units.temperature()),
                ("windspeed_10m", units.speed()),
                ("timezone", "auto".to_string()),
                ("forecast_days", "16".to_string()),
                ("hourly", params),
            ],
        )?;

        let resp = Client::new().get(url.to_string()).send().await?;
        if !resp.status().is_success() {
            return Err(anyhow!(
                "HTTP request to {} returned {}: {}",
                url,
                resp.status(),
                resp.text().await?
            ));
        }

        let data: Response = resp.json().await?;
        data.hourly.as_weather(when, name, location, units)
    }

    pub fn as_json(&self) -> Result<String> {
        Ok(serde_json::to_string(&self)?)
    }

    pub fn as_string(&self) -> Result<String> {
        let heading = match &self.name {
            Some(name) => format!(
                "{} {} ({}) {} {}\n",
                emoji::CALENDAR,
                name,
                self.date.format(DATE_OUTPUT_FORMAT),
                emoji::GLOBE,
                self.location
            ),
            None => "".to_string(),
        };
        let temperature = match self.units {
            Units::Metric => "C",
            Units::Imperial => "F",
        };
        let speed = match self.units {
            Units::Metric => "km/h",
            Units::Imperial => "mph",
        };

        Ok(format!(
            "{}{} {}°{} (feels like {}°{}) {} {}% chance of rain & {}% humidity {} {}{} {}",
            heading,
            emoji::emoji_for_weather(self.weather_code)?,
            self.temperature.round(),
            temperature,
            self.feels_like.round(),
            temperature,
            emoji::PRECIPITATION,
            self.probability_of_precipitation.unwrap_or(0),
            self.humidity,
            emoji::WIND,
            self.wind_speed.round(),
            speed,
            wind::wind_direction(self.wind_direction)?,
        ))
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    hourly: Hourly,
}

#[derive(Deserialize, Debug)]
struct Hourly {
    time: Vec<String>,
    temperature_2m: Vec<f64>,
    apparent_temperature: Vec<f64>,
    relativehumidity_2m: Vec<i8>,
    precipitation_probability: Vec<Option<i8>>,
    windspeed_10m: Vec<f64>,
    winddirection_10m: Vec<i32>,
    weathercode: Vec<i8>,
}

impl Hourly {
    fn as_weather(
        &self,
        target: NaiveDateTime,
        name: Option<String>,
        location: String,
        units: &Units,
    ) -> Result<Weather> {
        let dates: Vec<NaiveDateTime> = self
            .time
            .iter()
            .map(|t| NaiveDateTime::parse_from_str(t, OPEN_METEO_DATE_FORMAT))
            .collect::<Result<Vec<NaiveDateTime>, ParseError>>()?;
        let diffs: Vec<i64> = dates
            .iter()
            .map(|t| target - *t)
            .map(|t| t.num_minutes().abs())
            .collect();
        let min_diff = diffs.iter().min().ok_or(anyhow!("No weather data found"))?;
        let idx = diffs
            .iter()
            .position(|t| t == min_diff)
            .ok_or(anyhow!("No weather data found"))?;

        Ok(Weather {
            name,
            location,
            weather_code: self.weathercode[idx],
            icon: emoji_for_weather(self.weathercode[idx])?,
            units: units.clone(),
            date: dates[idx],
            probability_of_precipitation: self.precipitation_probability[idx],
            temperature: self.temperature_2m[idx],
            feels_like: self.apparent_temperature[idx],
            humidity: self.relativehumidity_2m[idx],
            wind_speed: self.windspeed_10m[idx],
            wind_direction: self.winddirection_10m[idx],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_units_temperature_metric() {
        let units = Units::Metric;
        assert_eq!(units.temperature(), "celsius");
    }

    #[test]
    fn test_units_temperature_imperial() {
        let units = Units::Imperial;
        assert_eq!(units.temperature(), "fahrenheit");
    }

    #[test]
    fn test_units_speed_metric() {
        let units = Units::Metric;
        assert_eq!(units.speed(), "kmh");
    }

    #[test]
    fn test_units_speed_imperial() {
        let units = Units::Imperial;
        assert_eq!(units.speed(), "mph");
    }

    #[test]
    fn test_weather_as_json() {
        let weather = Weather {
            name: Some("Event".to_string()),
            location: "Location".to_string(),
            units: Units::Metric,
            icon: "☀️".to_string(),
            date: NaiveDateTime::from_timestamp_opt(1621555200, 0).unwrap(),
            weather_code: 1,
            probability_of_precipitation: Some(20),
            temperature: 25.0,
            feels_like: 28.0,
            humidity: 80,
            wind_speed: 10.0,
            wind_direction: 180,
        };

        let result = weather.as_json();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            r#"{"name":"Event","location":"Location","units":"Metric","icon":"☀️","date":"2021-05-21 00:00:00","weather_code":1,"probability_of_precipitation":20,"temperature":25.0,"feels_like":28.0,"humidity":80,"wind_speed":10.0,"wind_direction":180}"#
        );
    }

    #[test]
    fn test_weather_as_string() {
        let weather = Weather {
            name: Some("Event".to_string()),
            location: "Location".to_string(),
            units: Units::Metric,
            icon: "☀️".to_string(),
            date: NaiveDateTime::from_timestamp_opt(1621555200, 0).unwrap(),
            weather_code: 1,
            probability_of_precipitation: Some(20),
            temperature: 25.0,
            feels_like: 28.0,
            humidity: 80,
            wind_speed: 10.0,
            wind_direction: 180,
        };

        let result = weather.as_string();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [
                "🗓️ Event (May 21, 00:00) 🌐 Location",
                "☀️ 25°C (feels like 28°C) ☔ 20% chance of rain & 80% humidity 💨 10km/h S"
            ]
            .join("\n")
        );
    }
}
