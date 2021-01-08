use clap::{value_t, App, Arg, SubCommand};
use serde::Deserialize;
// use std::fs;
// use std::path::PathBuf;
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

fn run() -> Result<(), Option<&'static str>> {
    let args = get_args();
    unsafe {
        VERBOSE = args.occurrences_of("v") > 0;
    }
    let cl_config = get_config_from_cl(&args);
    printlnv!("Command line config is {:?}.", cl_config);
    Ok(())
}

fn get_args<'a>() -> clap::ArgMatches<'a> {
    let app = get_args_app();
    app.get_matches()
}

fn get_args_app<'a, 'b>() -> App<'a, 'b> {
    App::new("speedtest")
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
        ("run", Some(run_args)) => Some(Config::Run {
            simulate: run_args.is_present("simulate"),
        }),
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
            Some(Config::Alert {
                email: alert_args.value_of("email").unwrap(),
                smtp: Smtp {
                    server: server,
                    port: port,
                    credentials: credentials,
                },
            })
        }
        _ => None,
    }
}

#[derive(Debug)]
enum Config<'a> {
    Run { simulate: bool },
    Alert { email: &'a str, smtp: Smtp<'a> },
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
