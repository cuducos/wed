# Weather on the Event Day [![Tests](https://github.com/cuducos/wed/actions/workflows/tests.yml/badge.svg)](https://github.com/cuducos/wed/actions/workflows/tests.yml) [![Linters](https://github.com/cuducos/wed/actions/workflows/linters.yml/badge.svg)](https://github.com/cuducos/wed/actions/workflows/linters.yml)

`wed`'s idea is to automate something I do pretty often: once I register for a run or triathlon event, I frequently check the city's weather forecast for the race's date and time in the previous days.

## Install

You can download the binary for your platform from the [releases page](https://github.com/cuducos/wed/releases), for example:

```console
$ curl -LO https://github.com/cuducos/wed/releases/download/v0.0.3/wed-x86_64-unknown-linux-gnu.tar.gz
$ tar -xzvf wed-x86_64-unknown-linux-gnu.tar.gz
$ rm wed-x86_64-unknown-linux-gnu.tar.gz
$ chmod a+x wed
$ mv wed /usr/local/bin/
```

### Compile from source

With [`cargo`](https://www.rust-lang.org/) installed:

```console
$ cargo install --path .
```

## Use cases

`wed` and its sub-commands that output weather information accept the options `--units`, `--json` and `--verbose` **before** the subcommand (e.g. `wed --json` or `wed --json forecast "Ottawa, ON" "2022-07-09 09:00"`).

Try `wed --help` for details.

### Weather forecast for any location, date and time

Run `wed forecast` with two arguments:

```console
$ wed forecast "Ottawa, ON" "2022-07-09 09:00"
🌤 26°C (feels like 27°C)  ☔ 40% chance of rain & 10% humidity 💨 4.2 km/h W
```

<details>

<summary>Or with a JSON output:</summary>

```console
$ wed --json forecast "Ottawa, ON" "2022-07-09 09:00"
{
    "name": null,
    "location": "Ottawa, CA",
    "units": "Metric",
    "icon": "\u26c5",
    "date": "2023-05-28 07:00:00",
    "weather_code": 3,
    "probability_of_precipitation": 13,
    "temperature": 17.4,
    "feels_like": 17.8,
    "humidity": 90,
    "wind_speed": 10.8,
    "wind_direction": 244
}
```

</details>

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

<details>

<summary>Or with a JSON output:</summary>

```console
$ wed --json
[
    {
        "name": null,
        "location": "Ottawa, CA",
        "units": "Metric",
        "icon": "\u26c5",
        "date": "2023-05-28 07:00:00",
        "weather_code": 3,
        "probability_of_precipitation": 13,
        "temperature": 17.4,
        "feels_like": 17.8,
        "humidity": 90,
        "wind_speed": 10.8,
        "wind_direction": 244
    }
]
```

</details>

### macOS notification

```console
$ wed notify
```

[![](https://i.imgur.com/0tZjOVS.png)](https://imgur.com/a/pitcf3k)

Running this command on a non-macOS system will output the same information as `wed --json notify`:

```console
$ wed --json notify
{
    "title": "🗓️ Formula Run (Jun 11, 09:00)",
    "subtitle": "🌐 Ottawa, ON, Canada",
    "body": "⛅ 18°C (feels like 17°C)\n☔ 0% chance of rain & 69% humidity\n💨 9km/h E"
}
```

### Delete a saved event

Run `wed delete` with one argument, the name of the event:

```console
$ wed delete "National Capital Triathlon"
```

### List all saved events

Run `wed list` with no arguments.

## Data

### Sources

* Conversion of location (city/country) to a latitude and longitude: [Nominatin](https://wiki.openstreetmap.org/wiki/Nominatim)
* Weather forecast, only when _t - 16 days_: [hourly from Open Meteo](https://open-meteo.com/en/docs)

### Persistence

* Saved events are saved in a simple file `~/.wed.json`
* The app automatically deletes past events when the app is run
* The app ignores events that are more than 16 days ahead
