# Speed test

[![Docker Stars](https://img.shields.io/docker/stars/giggio/speedtest.svg)](https://hub.docker.com/r/giggio/speedtest/)
[![Docker Pulls](https://img.shields.io/docker/pulls/giggio/speedtest.svg)](https://hub.docker.com/r/giggio/speedtest/)
[![ImageLayers](https://images.microbadger.com/badges/image/giggio/speedtest.svg)](https://microbadger.com/#/images/giggio/speedtest)

This app runs a speed test and saves the history in .json files and an aggregate .csv
file. It can also alert you with an e-mail if it finds that the bandwidth is bellow
an specified value.

This can be run on Linux for AMD64 and ARMv7.

## Upstream Links

* Docker Registry @ [giggio/speedtest](https://hub.docker.com/r/giggio/speedtest/)
* GitHub @ [giggio/speedtest](https://github.com/giggio/speedtest)

## Quick Start

You need to mount a volume to `/data`, and the files will be saved there.
Run it like this:

````bash
docker run --rm -ti -v `pwd`/data:/data giggio/speedtest run
````

After running will have a .json file with a date/time structure
(e.g. 202011212124.json) and a `speed.csv` file.

## Add a cron

To have a history a good idea is to add a cron job (with `crontab -e`) like
this:

````cron
0 */3 * * * docker run --rm -ti -v /path/to/my/data:/data giggio/speedtest run
````

### Detailed commands

There are two commands: `run` and `alert`. The former runs the speed test, the
second alerts you for a bandwidth bellow specification.

All commands have a `-v` option for verbose output, and you can get help by
running `docker run --rm giggio/speedtest --help`.

#### Running a speed test

To view available args run:

````bash
docker run --rm giggio/speedtest run --help
````

This command has a simulated argument, which will make it simply drop some results
into the data folder. It is useful to help you setup your infrastructure without
having to wait for a full speed test to run and also does not use any bandwidth.
It simply saves the files.

#### Alerting

To view available args run:

````bash
docker run --rm giggio/speedtest alert --help
````

This command will send you an e-mail using SMTP. You have to supply the values
like server, port, sender and destination e-mail addresses etc. Authentication
information is optional, but most mail servers will require it.

It also has a simulate argument. It will not send the email, but simply write to
the terminal on stdout what it would send through in an e-mail.

It will take the last 8 results (customizable with `--count`) and average them.

You need to supply expected the upload and download bandwidth, and you may
optionally supply a threshold to when the e-mail should be sent (defaults to 20%).

## Background

This project was previosly made up of a few bash scripts and a Node.js tool
to measure the results.
This new version is written in Rust and is using the official
[CLI from Ookla](https://www.speedtest.net/apps/cli). It is much faster
(Rust <3) and, due to using the official Ookla CLI, more acurate. The
container is also much smaller, simply containing the binaries, written from
scratch, without any distro files.

Ookla's tool does not support a some of the information that the Node.js tool
supported (server latitude, longitude, distance and server ping). It still
supplies the most important values, like upload and download bandwidth, ping
latency, ISP, server host, city and country. The columns in the CSV file that
had that information are now null and will be eventually removed.

Also, the .json files format is now in a different format from before.

## Contributing

Questions, comments, bug reports, and pull requests are all welcome.  Submit them at
[the project on GitHub](https://github.com/giggio/speedtest/).

Bug reports that include steps-to-reproduce (including code) are the
best. Even better, make them in the form of pull requests.

## Author

[Giovanni Bassi](https://github.com/giggio)

## License

Licensed under the MIT license.