# Metro Schedule API
[![License](http://img.shields.io/:license-mit-blue.svg)](http://badges.mit-license.org)
[![Donate](https://img.shields.io/badge/Donate-PayPal-green.svg)](https://paypal.me/bryantdeters)

A simple JSON web API to get the arrival time for a train in the St. Louis Metro given a destination station and direction.

Request: POST localhost:8000/next-arrival
```json
{
	"station":"cwe",
	"direction":"west"
}
```

Response:
```json
{
	"station": "cwe",
	"direction": "west",
	"time": "9:44P"
}
```

Note that this is not Japan and that trains may be late/early by ±2 minutes or more. 

This API is intended for timing your arrival to a Metrolink station in an optimal way. For more advanced route planning with public transit, see Google Maps or [Moovit](https://moovit.com/).

A valid ticket or pass is required to ride the Metro trains.

## Getting Started: 
Install [Rust](https://www.rust-lang.org/en-US/) in order to compile this project

* On windows: follow the link above and download installer

* On pretty much anything else run the following command:

```
curl https://sh.rustup.rs -sSf | sh
```

Once Rust and its dependency manager, `cargo` are installed, compile the project with the following command:

```
cargo build --release
```

## Deployment

I personally have this running on a [raspberry pi](https://www.raspberrypi.org) along with my [telegram chat bot](https://github.com/brunt/telegram-bot).
 
 Dockerization and cloud deployment via kubernetes or docker swarm are certainly possible but not necessary for my personal use case.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Acknowledgments

* Train schedule csv files were created from data available from the St. Louis [Metrolink System Schedule](https://www.metrostlouis.org/metrolink-system-schedule/)
* [Telegram](https://telegram.org/) is super helpful as it gives me a mobile UI and chat bot to interact with this API for $0.00.

