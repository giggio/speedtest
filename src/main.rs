use chrono::{DateTime, Utc};
use clap::{App, Arg, SubCommand};
use serde::Deserialize;
//use serde_json::Result;
use std::process::Command;
use which::which;
mod macros;

static mut VERBOSE: bool = false;

fn main() {
    match run() {
        Err(None) => std::process::exit(1),
        Err(Some(x)) => {
            eprintln!("{}", x);
            std::process::exit(1);
        }
        Ok(_) => std::process::exit(0),
    }
}

fn run() -> Result<(), Option<String>> {
    let args = get_args();
    unsafe {
        VERBOSE = args.occurrences_of("v") > 0;
    }
    let cl_config = get_config_from_cl(&args);
    printlnv!("Command line config is {:?}.", cl_config);
    match cl_config {
        Some(config) => match config {
            Config::Run(run) => command_run(run),
            Config::Alert(alert) => command_alert(alert),
        },
        _ => Err(None),
    }
}

fn command_run(run: Run) -> Result<(), Option<String>> {
    let result = if run.simulate {
        SpeedResult {
            client_ip: String::from("15.22.77.1"),
            client_isp: String::from("Some ISP"),
            date: Utc::now(),
            download: 123.45,
            upload: 65.43,
            ping: 5.7279999999999998,
            server_country: String::from("Brazil"),
            server_host: String::from("Some host"),
            server_id: 99999,
            server_location: String::from("Diadema"),
            jsonresult: String::from(
                r#"{"type":"result","timestamp":"2021-01-03T12:10:00Z","ping":{"jitter":0.28499999999999998,"latency":5.7279999999999998},"download":{"bandwidth":20309419,"bytes":176063552,"elapsed":8815},"upload":{"bandwidth":13206885,"bytes":195610380,"elapsed":15015},"packetLoss":0,"isp":"Some ISP","interface":{"internalIp":"192.168.1.2","name":"eth0","macAddr":"99:99:99:99:99:99","isVpn":false,"externalIp":"84.6.0.1"},"server":{"id":99999,"name":"Some Server","location":"São Paulo","country":"Brazil","host":"someserver.nonexistentxyz.com","port":10000,"ip":"15.22.77.1"},"result":{"id":"babad438-ac4b-47db-bc28-2de7e257bd28","url":"https://www.fakespeedtest.net/result/c/babad438-ac4b-47db-bc28-2de7e257bd28"}}"#,
            ),
        }
    } else {
        match which("speedtest") {
            Ok(speedtestbin) => {
                let child = Command::new(speedtestbin)
                    .args(&[
                        "--accept-license",
                        "--accept-gdpr",
                        "--format=json",
                        "--progress=no",
                    ])
                    .spawn()
                    .unwrap();
                let output = child.wait_with_output().unwrap();
                let speed_result = if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    match convert_json(stdout.to_string()) {
                        Ok(result) => result,
                        Err(err) => return Err(Some(err)),
                    }
                } else {
                    let msg = format!(
                        "Speedtest executable returned an error. Output:\n{}\nErrors:\n{}",
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    return Err(Some(msg));
                };
                speed_result
            }
            Err(_) => return Err(Some("Could not find speedtest binary.".to_owned())),
        }
    };
    Ok(())
}

fn convert_json(json: String) -> Result<SpeedResult, String> {
    let result: serde_json::Result<RawSpeedResult> = serde_json::from_str(&json);
    match result {
        Ok(raw_result) => Ok(SpeedResult {
            client_ip: raw_result.interface.externalIp,
            client_isp: raw_result.isp,
            date: Utc::now(),
            download: raw_result.download.bandwidth,
            upload: raw_result.upload.bandwidth,
            ping: raw_result.ping.latency,
            server_country: raw_result.server.country,
            server_host: raw_result.server.host,
            server_id: raw_result.server.id,
            server_location: raw_result.server.location,
            jsonresult: json,
        }),
        Err(err) => {
            let msg = format!(
                "Could not parse result. Json:\n{}\nError:{}",
                String::from_utf8_lossy(&json.as_bytes()),
                err
            );
            Err(msg)
        }
    }
}

fn command_alert(alert: Alert) -> Result<(), Option<String>> {
    unimplemented!();
}

fn get_args<'a>() -> clap::ArgMatches<'a> {
    let app = get_args_app();
    app.get_matches()
}

fn get_args_app<'a, 'b>() -> App<'a, 'b> {
    App::new("trackspeedtest")
        .version("0.1")
        .author("Giovanni Bassi <giggio@giggio.net>")
        .about("Runs speed test and adds results to file and alert if necessary")
        .arg(
            Arg::with_name("v")
                .short("v")
                .long("verbose")
                .global(true)
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            SubCommand::with_name("alert")
                .about("Sends an email if last average is bellow an bandwith value")
                .arg(
                    Arg::with_name("email")
                        .short("e")
                        .long("email")
                        .takes_value(true)
                        .index(1)
                        .required(true)
                        .help("E-mail address to send the alert message to"),
                )
                .arg(
                    Arg::with_name("smtp server")
                        .short("s")
                        .long("smtp")
                        .takes_value(true)
                        .index(2)
                        .required(true)
                        .help("SMTP server and port to use, use server:port")
                        .validator(|server_and_port| {
                            let parts: Vec<&str> = server_and_port.split(':').collect();
                            if parts.len() != 2 {
                                return Err("Not valid server".to_owned());
                            }
                            if let Err(_) = parts[1].parse::<u16>() {
                                return Err("Port is not in the correct format.".to_owned());
                            }
                            Ok(())
                        }),
                )
                .arg(
                    Arg::with_name("username")
                        .short("u")
                        .long("username")
                        .takes_value(true)
                        .help("SMTP server user for authentication"),
                )
                .arg(
                    Arg::with_name("password")
                        .short("p")
                        .long("password")
                        .takes_value(true)
                        .help("SMTP server password for authentication"),
                )
                .arg(
                    Arg::with_name("hours")
                        .short("H")
                        .long("hours")
                        .default_value("24")
                        .help("Last hours to use as average"),
                ),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Runs the speed test.")
                .arg(
                    Arg::with_name("simulate")
                        .short("s")
                        .long("simulate")
                        .help("Should simulate instead of running speed test"),
                ),
        )
}

fn get_config_from_cl<'a>(args: &'a clap::ArgMatches) -> Option<Config<'a>> {
    match args.subcommand() {
        ("run", Some(run_args)) => Some(Config::Run(Run {
            simulate: run_args.is_present("simulate"),
        })),
        ("alert", Some(alert_args)) => {
            let server_and_port = alert_args
                .value_of("smtp server")
                .expect("Should have server as it is required");
            let parts: Vec<&str> = server_and_port.split(':').collect();
            let server = parts[0];
            let port = parts[1].parse::<u16>().unwrap();
            let credentials;
            if let (Some(username), Some(password)) = (
                alert_args.value_of("username"),
                alert_args.value_of("password"),
            ) {
                credentials = Some(Credentials {
                    username: username,
                    password: password,
                });
            } else {
                credentials = None;
            }
            Some(Config::Alert(Alert {
                email: alert_args.value_of("email").unwrap(),
                smtp: Smtp {
                    server: server,
                    port: port,
                    credentials: credentials,
                },
            }))
        }
        _ => None,
    }
}

#[derive(Debug)]
enum Config<'a> {
    Run(Run),
    Alert(Alert<'a>),
}
#[derive(Debug)]
struct Run {
    simulate: bool,
}
#[derive(Debug)]
struct Alert<'a> {
    email: &'a str,
    smtp: Smtp<'a>,
}

#[derive(Debug)]
struct Smtp<'a> {
    server: &'a str,
    port: u16,
    credentials: Option<Credentials<'a>>,
}

#[derive(Debug)]
struct Credentials<'a> {
    username: &'a str,
    password: &'a str,
}

#[derive(Debug)]
struct SpeedResult {
    date: DateTime<Utc>,
    ping: f64,
    download: f64,
    upload: f64,
    client_ip: String,
    client_isp: String,
    server_host: String,
    server_location: String,
    server_country: String,
    server_id: u32,
    jsonresult: String,
}

// HEADER='date,ping,speeds_download,speeds_upload,client_ip,client_isp,server_host,server_lat,server_lon,server_location,server_country,location_distance,server_ping,server_id'

#[derive(Deserialize)]
struct RawSpeedResult {
    // | jq '.ping.latency,(.download.bandwidth*8/1024/1024*100|round/100),(.upload.bandwidth*8/1024/1024*100|round/100),.interface.externalIp,.isp,.server.host,null,null,.server.location,.server.country,null,null,.server.id' \
    ping: RawPing,
    download: RawBandwidth,
    upload: RawBandwidth,
    interface: RawInterface,
    isp: String,
    server: RawServer,
}
#[derive(Deserialize)]
struct RawPing {
    latency: f64,
}
#[derive(Deserialize)]
struct RawBandwidth {
    bandwidth: f64,
}
#[derive(Deserialize)]
struct RawInterface {
    externalIp: String,
}
#[derive(Deserialize)]
struct RawServer {
    host: String,
    location: String,
    country: String,
    id: u32,
}
