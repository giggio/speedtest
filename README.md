# Speed test

[![Docker Stars](https://img.shields.io/docker/stars/giggio/speedtest.svg)](https://hub.docker.com/r/giggio/speedtest/)
[![Docker Pulls](https://img.shields.io/docker/pulls/giggio/speedtest.svg)](https://hub.docker.com/r/giggio/speedtest/)
[![ImageLayers](https://images.microbadger.com/badges/image/giggio/speedtest.svg)](https://microbadger.com/#/images/giggio/speedtest)

This runs a speed test and saves the history in .json files and an aggregate .csv
file.

This can be run on Linux for AMD64, ARMv7 and ARM64.

## Upstream Links

* Docker Registry @ [giggio/speedtest](https://hub.docker.com/r/giggio/speedtest/)
* GitHub @ [giggio/speedtest](https://github.com/giggio/speedtest)

## Quick Start

You need to mount a volume to `/app/data`, and the files will be saved there.
Run it like this:

````bash
docker run --rm -ti -v `pwd`/data:/app/data giggio/speedtest
````

After running will have a .json file with a date/time structure
(e.g. 202011212124.json) and a `speed.csv` file.

## Contributing

Questions, comments, bug reports, and pull requests are all welcome.  Submit them at
[the project on GitHub](https://github.com/giggio/speedtest/).

Bug reports that include steps-to-reproduce (including code) are the
best. Even better, make them in the form of pull requests.

## Author

[Giovanni Bassi](https://github.com/giggio)

## License

Licensed under the MIT license.