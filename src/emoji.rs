use anyhow::{anyhow, Result};

pub const PRECIPITATION: &str = "☔";
pub const WIND: &str = "💨";
// pub const CALENDAR: &str = "🗓";
// pub const GLOBE: &str = "🌐";

pub fn emoji_for_weather(name: &str) -> Result<&str> {
    Ok(match name {
        "Thunderstorm" => "🌩️",
        "Drizzle" => "🌧️",
        "Rain" => "🌧️",
        "Snow" => "🌨️",
        "Clear" => "☀️",
        "Atmosphere" => "☁️",
        "Clouds" => "⛅",
        _ => return Err(anyhow!("Unknown weather: {}", name)),
    })
}
