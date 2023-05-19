use anyhow::{anyhow, Result};

static DEGREES: &[i32] = &[0, 45, 90, 135, 180, 225, 270, 315, 360];
static DIRECTIONS: &[&str] = &["N", "NE", "E", "SE", "S", "SW", "W", "NW", "N"];

pub fn wind_direction(deg: i32) -> Result<String> {
    let closest = DEGREES.iter().min_by_key(|d| (deg - *d).abs());

    match closest {
        None => Err(anyhow!("No closest direction found for {}", deg)),
        Some(degree) => {
            let idx = DEGREES.iter().position(|d| d == degree).unwrap();
            Ok(DIRECTIONS[idx].to_string())
        }
    }
}
