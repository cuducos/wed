use anyhow::{anyhow, Result};

static DEGREES: &[f64] = &[0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0, 360.0];
static DIRECTIONS: &[&str] = &["N", "NE", "E", "SE", "S", "SW", "W", "NW", "N"];

pub fn wind_direction(deg: f64) -> Result<String> {
    let closest = DEGREES
        .iter()
        .min_by_key(|d| (deg - *d).abs().round() as i32);

    match closest {
        None => Err(anyhow!("No closest direction found for {}", deg)),
        Some(degree) => {
            let idx = DEGREES.iter().position(|d| d == degree).unwrap();
            Ok(DIRECTIONS[idx].to_string())
        }
    }
}
