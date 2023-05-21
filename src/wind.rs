use anyhow::{anyhow, Result};

static DEGREES: &[i32] = &[0, 45, 90, 135, 180, 225, 270, 315, 360];
static DIRECTIONS: &[&str] = &["N", "NE", "E", "SE", "S", "SW", "W", "NW", "N"];

pub fn wind_direction(deg: i32) -> Result<String> {
    if !(0..=360).contains(&deg) {
        return Err(anyhow!(
            "Wind direction outside of the range 0..360: {}",
            deg
        ));
    }

    let closest = DEGREES.iter().min_by_key(|d| (deg - *d).abs());
    match closest {
        None => Err(anyhow!("No closest direction found for {}", deg)),
        Some(degree) => {
            let idx = DEGREES.iter().position(|d| d == degree).unwrap();
            Ok(DIRECTIONS[idx].to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wind_direction_north() {
        let deg = 20;
        let result = wind_direction(deg).unwrap();
        assert_eq!(result, "N");
    }

    #[test]
    fn test_wind_direction_northeast() {
        let deg = 30;
        let result = wind_direction(deg).unwrap();
        assert_eq!(result, "NE");
    }

    #[test]
    fn test_wind_direction_east() {
        let deg = 80;
        let result = wind_direction(deg).unwrap();
        assert_eq!(result, "E");
    }

    #[test]
    fn test_wind_direction_southeast() {
        let deg = 135;
        let result = wind_direction(deg).unwrap();
        assert_eq!(result, "SE");
    }

    #[test]
    fn test_wind_direction_south() {
        let deg = 180;
        let result = wind_direction(deg).unwrap();
        assert_eq!(result, "S");
    }

    #[test]
    fn test_wind_direction_southwest() {
        let deg = 225;
        let result = wind_direction(deg).unwrap();
        assert_eq!(result, "SW");
    }

    #[test]
    fn test_wind_direction_west() {
        let deg = 270;
        let result = wind_direction(deg).unwrap();
        assert_eq!(result, "W");
    }

    #[test]
    fn test_wind_direction_northwest() {
        let deg = 315;
        let result = wind_direction(deg).unwrap();
        assert_eq!(result, "NW");
    }

    #[test]
    fn test_wind_direction_invalid() {
        let deg = 400;
        let result = wind_direction(deg);
        assert!(result.is_err());
    }
}
