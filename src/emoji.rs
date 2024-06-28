use anyhow::{anyhow, Result};

pub const PRECIPITATION: &str = "☔";
pub const WIND: &str = "💨";
pub const CALENDAR: &str = "🗓️";
pub const GLOBE: &str = "🌐";

pub fn emoji_for_weather<'a>(code: i8) -> Result<&'a str> {
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_for_weather_sunny() {
        let code = 0;
        let result = emoji_for_weather(code).unwrap();
        assert_eq!(result, "☀️");
    }

    #[test]
    fn test_emoji_for_weather_partly_cloudy() {
        let code = 3;
        let result = emoji_for_weather(code).unwrap();
        assert_eq!(result, "⛅");
    }

    #[test]
    fn test_emoji_for_weather_cloudy() {
        let code = 46;
        let result = emoji_for_weather(code).unwrap();
        assert_eq!(result, "☁️");
    }

    #[test]
    fn test_emoji_for_weather_rainy() {
        let code = 55;
        let result = emoji_for_weather(code).unwrap();
        assert_eq!(result, "🌧️");
    }

    #[test]
    fn test_emoji_for_weather_snowy() {
        let code = 75;
        let result = emoji_for_weather(code).unwrap();
        assert_eq!(result, "🌨️");
    }

    #[test]
    fn test_emoji_for_weather_thunderstorm() {
        let code = 97;
        let result = emoji_for_weather(code).unwrap();
        assert_eq!(result, "🌩️");
    }

    #[test]
    fn test_emoji_for_weather_unknown_code() {
        let code = 100;
        let result = emoji_for_weather(code);
        assert!(result.is_err());
    }
}
