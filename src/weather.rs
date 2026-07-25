use anyhow::{anyhow, Result};
use chrono::NaiveDateTime;
use reqwest::Url;
use serde::{self, Deserialize, Serialize};

use crate::date_format::{self, OPEN_METEO_DATE_FORMAT};
use crate::emoji::{self, emoji_for_weather};
use crate::units::Units;
use crate::wind;
use crate::WedClient;

const API_URL: &str = "https://api.open-meteo.com/v1/forecast";
const DATE_OUTPUT_FORMAT: &str = "%b %-d, %H:%M";

#[derive(Serialize, Debug)]
pub struct Notification {
    pub title: String,
    pub subtitle: String,
    pub body: String,
}

#[derive(Serialize, Debug)]
pub struct HourlyForecast<'a> {
    pub icon: &'a str,
    #[serde(with = "date_format")]
    pub date: NaiveDateTime,
    pub weather_code: i8,
    pub probability_of_precipitation: i8,
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: i8,
    pub wind_speed: f64,
    pub wind_direction: i32,
}

pub struct Window {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

#[derive(Serialize, Debug)]
pub struct Weather<'a> {
    pub name: Option<String>,
    pub location: String,
    pub units: Units,
    pub icon: &'a str,

    #[serde(with = "date_format")]
    pub date: NaiveDateTime,
    pub weather_code: i8,
    pub probability_of_precipitation: i8,
    pub temperature: f64,
    pub feels_like: f64,
    pub humidity: i8,
    pub wind_speed: f64,
    pub wind_direction: i32,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub forecast: Vec<HourlyForecast<'a>>,
}

impl Weather<'_> {
    pub async fn new(
        client: &WedClient,
        event: &crate::Event,
        window: Option<Window>,
        units: &Units,
    ) -> Result<Self> {
        let (start, end) = match &window {
            Some(w) => (w.start, w.end),
            None => (event.when, event.when),
        };
        let start_date = start.format("%Y-%m-%d").to_string();
        let end_date = end.format("%Y-%m-%d").to_string();

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
                ("latitude", event.latitude.to_string()),
                ("longitude", event.longitude.to_string()),
                ("start_date", start_date),
                ("end_date", end_date),
                ("temperature_unit", units.temperature()),
                ("wind_speed_unit", units.speed()),
                ("timezone", "auto".to_string()),
                ("hourly", params),
            ],
        )?;

        let resp = client.get(url).await?;

        let url_for_error = resp.url().clone();
        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await?;
            return Err(anyhow!(
                "HTTP request to {} returned {}: {}",
                url_for_error,
                status,
                body
            ));
        }

        let data: Response = resp.json().await.map_err(|e| {
            let message = format!("Failed to parse response JSON body from {url_for_error}: {e}");
            anyhow!(message)
        })?;
        data.hourly.as_weather(
            event.name.clone(),
            event.when,
            window,
            event.location.clone(),
            units,
        )
    }

    pub fn as_notification(&self) -> Result<Notification> {
        let title = match &self.name {
            Some(name) => format!(
                "{} {} ({})",
                emoji::CALENDAR,
                name,
                self.date.format(DATE_OUTPUT_FORMAT),
            ),
            None => "".to_string(),
        };
        let subtitle = format!("{} {}", emoji::GLOBE, self.location);
        let temperature = match self.units {
            Units::Metric => "C",
            Units::Imperial => "F",
        };
        let speed = match self.units {
            Units::Metric => "km/h",
            Units::Imperial => "mph",
        };
        let body = format!(
            "{} {}°{} (feels like {}°{})\n{} {}% chance of rain & {}% humidity\n{} {}{} {}",
            emoji::emoji_for_weather(self.weather_code)?,
            self.temperature.round(),
            temperature,
            self.feels_like.round(),
            temperature,
            emoji::PRECIPITATION,
            self.probability_of_precipitation,
            self.humidity,
            emoji::WIND,
            self.wind_speed.round(),
            speed,
            wind::wind_direction(self.wind_direction)?,
        );

        Ok(Notification {
            title,
            subtitle,
            body,
        })
    }
    pub fn as_string(&self, json: bool, chart: bool, width: Option<usize>) -> Result<String> {
        if json {
            return Ok(serde_json::to_string(&self)?);
        }

        let notification = self.as_notification()?;
        let mut text = if chart {
            format!("{}\n", notification.subtitle)
        } else {
            format!(
                "{} {}\n{}",
                notification.title,
                notification.subtitle,
                notification.body.replace('\n', " "),
            )
        };

        if chart && !self.forecast.is_empty() {
            text.push_str(&crate::chart::render(self, width));
        }

        Ok(text)
    }
}

#[derive(Deserialize, Debug)]
struct Response {
    hourly: Hourly,
}

#[derive(Debug)]
struct HourlyItem {
    time: NaiveDateTime,
    temperature_2m: f64,
    apparent_temperature: f64,
    relativehumidity_2m: i8,
    precipitation_probability: i8,
    windspeed_10m: f64,
    winddirection_10m: i32,
    weathercode: i8,
}

#[derive(Deserialize, Debug)]
struct Hourly {
    time: Vec<String>,
    temperature_2m: Vec<Option<f64>>,
    apparent_temperature: Vec<Option<f64>>,
    relativehumidity_2m: Vec<Option<i8>>,
    precipitation_probability: Vec<Option<i8>>,
    windspeed_10m: Vec<Option<f64>>,
    winddirection_10m: Vec<Option<i32>>,
    weathercode: Vec<Option<i8>>,
}

impl Hourly {
    fn item(&self, idx: usize) -> Option<HourlyItem> {
        Some(HourlyItem {
            time: NaiveDateTime::parse_from_str(&self.time[idx], OPEN_METEO_DATE_FORMAT).ok()?,
            temperature_2m: self.temperature_2m[idx]?,
            apparent_temperature: self.apparent_temperature[idx]?,
            relativehumidity_2m: self.relativehumidity_2m[idx]?,
            precipitation_probability: self.precipitation_probability[idx].unwrap_or(0),
            windspeed_10m: self.windspeed_10m[idx]?,
            winddirection_10m: self.winddirection_10m[idx]?,
            weathercode: self.weathercode[idx]?,
        })
    }

    fn as_weather<'a>(
        &self,
        name: Option<String>,
        target: NaiveDateTime,
        window: Option<Window>,
        location: String,
        units: &Units,
    ) -> Result<Weather<'a>> {
        let item: HourlyItem = (0..self.time.len())
            .filter_map(|idx| self.item(idx))
            .min_by_key(|item| (target - item.time).num_minutes().abs())
            .ok_or(anyhow!("No weather data found"))?;

        let forecast: Result<Vec<HourlyForecast>> = match window {
            Some(w) => (0..self.time.len())
                .filter_map(|idx| self.item(idx))
                .filter(|i| i.time >= w.start && i.time <= w.end)
                .map(|i| {
                    Ok(HourlyForecast {
                        icon: emoji_for_weather(i.weathercode)?,
                        date: i.time,
                        weather_code: i.weathercode,
                        probability_of_precipitation: i.precipitation_probability,
                        temperature: i.temperature_2m,
                        feels_like: i.apparent_temperature,
                        humidity: i.relativehumidity_2m,
                        wind_speed: i.windspeed_10m,
                        wind_direction: i.winddirection_10m,
                    })
                })
                .collect(),
            None => Ok(vec![]),
        };

        Ok(Weather {
            name,
            location,
            weather_code: item.weathercode,
            icon: emoji_for_weather(item.weathercode)?,
            units: units.clone(),
            date: item.time,
            probability_of_precipitation: item.precipitation_probability,
            temperature: item.temperature_2m,
            feels_like: item.apparent_temperature,
            humidity: item.relativehumidity_2m,
            wind_speed: item.windspeed_10m,
            wind_direction: item.winddirection_10m,
            forecast: forecast?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weather_as_notification() {
        let weather = Weather {
            name: Some("Event".to_string()),
            location: "Location".to_string(),
            units: Units::Metric,
            icon: "☀️",
            date: NaiveDateTime::parse_from_str("2021-05-20 8:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            weather_code: 1,
            probability_of_precipitation: 20,
            temperature: 25.0,
            feels_like: 28.0,
            humidity: 80,
            wind_speed: 10.0,
            wind_direction: 180,
            forecast: vec![],
        };

        let result = weather.as_notification();
        assert!(result.is_ok());

        let notification = result.unwrap();
        assert_eq!(notification.title, "🗓️ Event (May 20, 08:00)");
        assert_eq!(notification.subtitle, "🌐 Location");

        let lines = notification.body.split('\n').collect::<Vec<&str>>();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "☀️ 25°C (feels like 28°C)");
        assert_eq!(lines[1], "☔ 20% chance of rain & 80% humidity");
        assert_eq!(lines[2], "💨 10km/h S");
    }

    #[test]
    fn test_weather_as_string() {
        let weather = Weather {
            name: Some("Event".to_string()),
            location: "Location".to_string(),
            units: Units::Metric,
            icon: "☀️",
            date: NaiveDateTime::parse_from_str("2021-05-20 8:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            weather_code: 1,
            probability_of_precipitation: 20,
            temperature: 25.0,
            feels_like: 28.0,
            humidity: 80,
            wind_speed: 10.0,
            wind_direction: 180,
            forecast: vec![],
        };

        let result = weather.as_string(false, false, None);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            [
                "🗓️ Event (May 20, 08:00) 🌐 Location",
                "☀️ 25°C (feels like 28°C) ☔ 20% chance of rain & 80% humidity 💨 10km/h S"
            ]
            .join("\n")
        );
    }

    #[test]
    fn test_weather_as_json() {
        let weather = Weather {
            name: Some("Event".to_string()),
            location: "Location".to_string(),
            units: Units::Metric,
            icon: "☀️",
            date: NaiveDateTime::parse_from_str("2021-05-20 8:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
            weather_code: 1,
            probability_of_precipitation: 20,
            temperature: 25.0,
            feels_like: 28.0,
            humidity: 80,
            wind_speed: 10.0,
            wind_direction: 180,
            forecast: vec![],
        };

        let result = weather.as_string(true, false, None);
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            r#"{"name":"Event","location":"Location","units":"Metric","icon":"☀️","date":"2021-05-20 08:00:00","weather_code":1,"probability_of_precipitation":20,"temperature":25.0,"feels_like":28.0,"humidity":80,"wind_speed":10.0,"wind_direction":180}"#
        );
    }
}
