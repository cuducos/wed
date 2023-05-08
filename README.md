# Weather on the Event Day [![Tests](https://github.com/cuducos/wed/actions/workflows/tests.yml/badge.svg)](https://github.com/cuducos/wed/actions/workflows/tests.yml) [![Linters](https://github.com/cuducos/wed/actions/workflows/linters.yml/badge.svg)](https://github.com/cuducos/wed/actions/workflows/linters.yml)

`wed`'s idea is to automate something I do pretty often: once I register for a run or triathlon event, I frequently check the city's weather forecast for the race's date and time in the previous days.

## Requirements

You will need an [OpenWeather API key](https://home.openweathermap.org/api_keys) as en environment variable called `OPEN_WEATHER_API_KEY`.

## Install

With [`cargo`](https://www.rust-lang.org/) installed:

```console
$ cargo install --path .
```

## Use cases

`wed` and its sub-commands that output weather information accepts the options `--units`, `--json` and `--verbose` **before** the subcommand (e.g. `wed --json` or `wed --json forecast "Ottawa, ON" "2022-07-09 09:00"`).

Try `wed --help` for datails.

### Weather forecast for any location, date and time

Run `wed forecast` with two arguments:

```console
$ wed forecast "Ottawa, ON" "2022-07-09 09:00"
🌤 26°C (feels like 27°C)  ☔ 40% chance of rain & 10% humidity 💨 4.2 km/h W
```

Or with a JSON output:

```console
$ wed --json forecast "Ottawa, ON" "2022-07-09 09:00"
{
    "name": null,
    "location": "Ottawa, ON",
    "units": "Metric",
    "date": "2022-07-09 09:00:00",
    "title": "Clear",
    "description": "clear sky",
    "probability_of_precipitation": 0.4,
    "temperature": 24.96,
    "feels_like": 22.75,
    "humidity": 67.0,
    "wind_speed": 2.62,
    "wind_direction": 283.0
}
```

### Save an event

Run `wed save` with three arguments:

```console
$ wed save "National Capital Triathlon" "Ottawa, ON" "2022-07-09 09:00"
🌤 26°C (feels like 27°C)  ☔ 40% chance of rain & 10% humidity 💨 4.2 km/h W
```

### Weather forecast for saved events

Run `wed` with no sub-command or arguments:

```console
$ wed
🗓 National Capital Triathlon (Jul 7, 09:00) 🌐 Ottawa, ON, Canada
🌤 26°C (feels like 27°C) ☔ 40% chance of rain & 10% humidity 💨 4.2 km/h W
```

Or with a JSON output:

```console
$ wed --json
[
    {
        "name": null,
        "location": "Ottawa, ON",
        "units": "Metric",
        "date": "2022-07-09 09:00:00",
        "title": "Clear",
        "description": "clear sky",
        "probability_of_precipitation": 0.4,
        "temperature": 24.96,
        "feels_like": 22.75,
        "humidity": 67.0,
        "wind_speed": 2.62,
        "wind_direction": 283.0
    }
]
```

### List all saved events

Run `wed list` with no arguments.

### Delete a saved event

Run `wed delete` with one argument, the name of the event:

```console
$ wed delete "National Capital Triathlon"
```

## Data sources

* Convertion of location (city/country) to a latitude and longitude: [Nominatin](https://wiki.openstreetmap.org/wiki/Nominatim)
* Weather forecast:
   * If _t - 5 days_: [3h forecast](https://openweathermap.org/forecast5)
   * There are other endpoints (hourly closer to the event, daily up to 30 days from the event), but since they are paid, `wed` doesn't use them

## Data management and persistence

* The app automaticaly deletes past events when the app is run
* The app igonres events that are more than 5 days ahead
* Data is persisted in simple file `~/.wed`
