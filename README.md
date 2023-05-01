# Weather on the Event Day [![Tests](https://github.com/cuducos/wed/actions/workflows/tests.yml/badge.svg)](https://github.com/cuducos/wed/actions/workflows/tests.yml) [![Linters](https://github.com/cuducos/wed/actions/workflows/linters.yml/badge.svg)](https://github.com/cuducos/wed/actions/workflows/linters.yml)

`wed`'s idea is to automate something I do pretty often: once I register for a run or triathlon event, I frequently check the city's weather forecast for the race's date and time in the previous days.

**:warning: THIS IS A WORK IN PROGRESS :warning:**

## TODO list

- Implement the JSON flag in the CLI
- Implement flags for unit and language
- Persist named events in `~/.wed`
- Automatically delete past events from `~/.wed` when the app is run

## Requirements

You will need an [OpenWeather API key](https://home.openweathermap.org/api_keys) as en environment variable called `OPEN_WEATHER_API_KEY`.

## Use cases

Try `wed --help`.

### Query a date and time followed by a location

Run `wed` with two arguments:

```console
$ wed --when "2022-07-09" --location "Ottawa, ON"
🌤 26°C (feels like 27°C)  ☔ 40% chance of rain & 10% humidity 💨 4.2 km/h W
```

Or with a JSON output (:warning: this is not implemented yet):

```console
$ wed --when "2022-07-09" --location "Ottawa, ON" --json
{
    "temperature": 26,
    "feels_like": 27,
    "chance_of_rain": 0.4,
    "humidity": 0.1,
    "wind_speed": 4.2,
    "wind_direction": "W",
    "description": "scattered clouds",
}
```

### Save an event (:warning: this is not implemented yet)

Run `wed` with three arguments:

```console
$ wed --name "National Capital Triathlon" --when "2022-07-09 09:00" --location "Ottawa, ON, Canada"
```

### Query saved events (:warning: this is not implemented yet)

Run `wed` with no arguments:

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
        "name": "National Capital Triathlon",
        "when": "2022-07-09 09:00:00",
        "location": "Ottawa, ON, Canada",
        "weather": {
            "temperature": 26,
            "feels_like": 27,
            "chance_of_rain": 0.4,
            "humidity": 0.1,
            "wind_speed": 4.2,
            "wind_direction": "W",
            "description": "scattered clouds",
        }
    }
]
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
