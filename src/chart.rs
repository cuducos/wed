use crate::units::Units;
use crate::weather::{HourlyForecast, Weather};

pub fn render(weather: &Weather, width_override: Option<usize>) -> String {
    let mut text = String::new();

    if weather.forecast.is_empty() {
        return text;
    }

    let mut min_temp = weather
        .forecast
        .iter()
        .map(|f| f.temperature.round() as i64)
        .min()
        .unwrap_or(0);
    let mut max_temp = weather
        .forecast
        .iter()
        .map(|f| f.temperature.round() as i64)
        .max()
        .unwrap_or(0);

    if min_temp == max_temp {
        min_temp -= 1;
        max_temp += 1;
    }

    let mut min_wind = weather
        .forecast
        .iter()
        .map(|f| f.wind_speed.round() as i64)
        .min()
        .unwrap_or(0);
    let mut max_wind = weather
        .forecast
        .iter()
        .map(|f| f.wind_speed.round() as i64)
        .max()
        .unwrap_or(0);

    if min_wind == max_wind {
        min_wind = (min_wind - 1).max(0);
        max_wind += 1;
    }

    let term = width_override.unwrap_or_else(|| {
        terminal_size::terminal_size()
            .map(|(terminal_size::Width(w), _)| w as usize)
            .unwrap_or(usize::MAX)
    });

    let columns = term.saturating_sub(10) / 7;
    let step = if columns > 0 {
        weather.forecast.len().div_ceil(columns)
    } else {
        1
    }
    .max(1);

    let forecast: Vec<&HourlyForecast> = weather.forecast.iter().step_by(step).collect();
    let width = forecast.len() * 7;

    let temp_unit = match weather.units {
        Units::Metric => "C",
        Units::Imperial => "F",
    };

    let wind_unit = match weather.units {
        Units::Metric => "km/h",
        Units::Imperial => "mph",
    };

    let height = (max_temp - min_temp + 1) as usize + 1;
    let mut grid = vec![vec![" ".to_string(); width]; height];

    let get_y = |temp: f64| -> usize {
        let t = temp.round() as i64;
        (max_temp - t) as usize + 1
    };

    for i in 0..forecast.len().saturating_sub(1) {
        let x1 = i * 7 + 3;
        let y1 = get_y(forecast[i].temperature);
        let x2 = (i + 1) * 7 + 3;
        let y2 = get_y(forecast[i + 1].temperature);

        let dx = x2 as isize - x1 as isize;
        let dy = y2 as isize - y1 as isize;

        for step in 1..dx {
            let x = x1 as isize + step;
            let y = y1 as isize + ((dy as f32 * step as f32) / dx as f32).round() as isize;
            let c = if dy < 0 {
                "/"
            } else if dy > 0 {
                "\\"
            } else {
                "-"
            };
            if y >= 0 && y < height as isize && x >= 0 && x < width as isize {
                grid[y as usize][x as usize] = c.to_string();
            }
        }
    }

    for (i, f) in forecast.iter().enumerate() {
        let x = i * 7 + 3;
        let y = get_y(f.temperature);

        if y >= 1 {
            grid[y - 1][x - 1] = f.icon.to_string();
            grid[y - 1][x] = "".to_string();
        }

        let temp_str = format!("{}°", f.temperature.round());
        let chars: Vec<char> = temp_str.chars().collect();
        let start_x = x.saturating_sub(chars.len() / 2);
        for (j, c) in chars.iter().enumerate() {
            if start_x + j < width {
                grid[y][start_x + j] = c.to_string();
            }
        }
    }

    text.push_str(&format!("\n Temp (°{})\n", temp_unit));
    for grid_row in grid.iter().take(height) {
        for cell in grid_row.iter().take(width) {
            text.push_str(cell);
        }
        text.push('\n');
    }

    let mut min_precip = weather
        .forecast
        .iter()
        .map(|f| (f.probability_of_precipitation as f32 / 5.0).round() as i64)
        .min()
        .unwrap_or(0);
    let mut max_precip = weather
        .forecast
        .iter()
        .map(|f| (f.probability_of_precipitation as f32 / 5.0).round() as i64)
        .max()
        .unwrap_or(0);

    if min_precip == max_precip {
        min_precip = (min_precip - 1).max(0);
        max_precip += 1;
    }

    let p_height = (max_precip - min_precip + 1) as usize + 1;
    let mut p_grid = vec![vec![" ".to_string(); width]; p_height];

    let get_py = |prob: i8| -> usize {
        let p = (prob as f32 / 5.0).round() as i64;
        (max_precip - p) as usize + 1
    };

    for i in 0..forecast.len().saturating_sub(1) {
        let x1 = i * 7 + 3;
        let y1 = get_py(forecast[i].probability_of_precipitation);
        let x2 = (i + 1) * 7 + 3;
        let y2 = get_py(forecast[i + 1].probability_of_precipitation);

        let dx = x2 as isize - x1 as isize;
        let dy = y2 as isize - y1 as isize;

        for step in 1..dx {
            let x = x1 as isize + step;
            let y = y1 as isize + ((dy as f32 * step as f32) / dx as f32).round() as isize;
            let c = if dy < 0 {
                "/"
            } else if dy > 0 {
                "\\"
            } else {
                "-"
            };
            if y >= 0 && y < p_height as isize && x >= 0 && x < width as isize {
                p_grid[y as usize][x as usize] = c.to_string();
            }
        }
    }

    for (i, f) in forecast.iter().enumerate() {
        let x = i * 7 + 3;
        let y = get_py(f.probability_of_precipitation);

        let p_str = format!("{}%", f.probability_of_precipitation);
        let chars: Vec<char> = p_str.chars().collect();
        let start_x = x.saturating_sub(chars.len() / 2);
        for (j, c) in chars.iter().enumerate() {
            if start_x + j < width {
                p_grid[y][start_x + j] = c.to_string();
            }
        }
    }

    let mut time_row = String::new();
    for f in &forecast {
        time_row.push_str(&format!("{:^7}", f.date.format("%H:%M")));
    }

    text.push_str(&"-".repeat(width));
    text.push('\n');
    text.push_str(&time_row);
    text.push('\n');
    text.push('\n');
    text.push_str(" Precipitation (%)\n");
    for grid_row in p_grid.iter().take(p_height) {
        for cell in grid_row.iter().take(width) {
            text.push_str(cell);
        }
        text.push('\n');
    }
    text.push_str(&"-".repeat(width));
    text.push('\n');
    text.push_str(&time_row);
    text.push('\n');
    text.push('\n');
    text.push_str(&format!(" Wind ({})\n", wind_unit));

    let w_height = (max_wind - min_wind + 1) as usize + 1;
    let mut w_grid = vec![vec![" ".to_string(); width]; w_height];

    let get_wy = |speed: f64| -> usize {
        let w = speed.round() as i64;
        (max_wind - w) as usize + 1
    };

    for i in 0..forecast.len().saturating_sub(1) {
        let x1 = i * 7 + 3;
        let y1 = get_wy(forecast[i].wind_speed);
        let x2 = (i + 1) * 7 + 3;
        let y2 = get_wy(forecast[i + 1].wind_speed);

        let dx = x2 as isize - x1 as isize;
        let dy = y2 as isize - y1 as isize;

        for step in 1..dx {
            let x = x1 as isize + step;
            let y = y1 as isize + ((dy as f32 * step as f32) / dx as f32).round() as isize;
            let c = if dy < 0 {
                "/"
            } else if dy > 0 {
                "\\"
            } else {
                "-"
            };
            if y >= 0 && y < w_height as isize && x >= 0 && x < width as isize {
                w_grid[y as usize][x as usize] = c.to_string();
            }
        }
    }

    for (i, f) in forecast.iter().enumerate() {
        let x = i * 7 + 3;
        let y = get_wy(f.wind_speed);

        if y >= 1 {
            let direction =
                crate::wind::wind_direction(f.wind_direction).unwrap_or_else(|_| "".to_string());
            let dir_chars: Vec<char> = direction.chars().collect();
            let start_x = x.saturating_sub(dir_chars.len() / 2);
            for (j, c) in dir_chars.iter().enumerate() {
                if start_x + j < width {
                    w_grid[y - 1][start_x + j] = c.to_string();
                }
            }
        }

        let w_str = format!("{}", f.wind_speed.round());
        let chars: Vec<char> = w_str.chars().collect();
        let start_x = x.saturating_sub(chars.len() / 2);
        for (j, c) in chars.iter().enumerate() {
            if start_x + j < width {
                w_grid[y][start_x + j] = c.to_string();
            }
        }
    }

    for grid_row in w_grid.iter().take(w_height) {
        for cell in grid_row.iter().take(width) {
            text.push_str(cell);
        }
        text.push('\n');
    }

    text.push_str(&"-".repeat(width));
    text.push('\n');
    text.push_str(&time_row);
    text.push('\n');

    text
}
