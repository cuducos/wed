use anyhow::{anyhow, Result};

pub const PRECIPITATION: &str = "☔";
pub const WIND: &str = "💨";
pub const CALENDAR: &str = "🗓️";
pub const GLOBE: &str = "🌐";

pub fn emoji_for_weather(code: i8) -> Result<String> {
    Ok(match code {
        0..=1 => "☀️",
        2..=3 => "⛅",
        45..=48 => "☁️",
        51..=67 => "🌧️",
        71..=77 => "🌨️",
        80..=82 => "🌧️",
        85..=86 => "🌨️",
        95..=99 => "🌩️",
        _ => return Err(anyhow!("Unknown weather code: {}", code)),
    }
    .to_string())
}
