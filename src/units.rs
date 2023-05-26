use clap::ValueEnum;
use serde::{self, Serialize};

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
}
