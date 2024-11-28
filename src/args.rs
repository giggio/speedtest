use clap::{App, AppSettings, Arg, SubCommand};

#[derive(Debug)]
pub struct Args {
    pub verbose: bool,
    pub command: Option<Command>,
}

#[derive(Debug)]
pub enum Command {
    Run(Run),
    Alert(Alert),
}

impl Args {
    pub fn new() -> Args {
        Args::new_from(&mut std::env::args_os()).unwrap_or_else(|err| err.exit())
    }
    fn new_from<I, T>(args: I) -> Result<Args, clap::Error>
    where
        I: Iterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        let args = Args::get_args_app().get_matches_from_safe(args)?;
        Ok(Args {
            verbose: args.occurrences_of("v") > 0,
            command: Args::get_config_from_cl(args),
        })
    }

    fn get_args_app<'a, 'b>() -> App<'a, 'b> {
        App::new("trackspeedtest")
            .version(env!("CARGO_PKG_VERSION"))
            .author("Giovanni Bassi <giggio@giggio.net>")
            .about("Runs and manages speed tests")
            .setting(AppSettings::ArgRequiredElseHelp)
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
                    .about("Sends an e-mail message if the average of the last measurements is bellow a bandwith value")
                    .arg(
                        Arg::with_name("sender email")
                            .takes_value(true)
                            .index(1)
                            .required(true)
                            .help("E-mail address to send the alert message from"),
                    )
                    .arg(
                        Arg::with_name("email")
                            .takes_value(true)
                            .index(2)
                            .required(true)
                            .help("E-mail address to send the alert message to"),
                    )
                    .arg(
                        Arg::with_name("smtp server")
                            .long("smtp")
                            .takes_value(true)
                            .index(3)
                            .required(true)
                            .help("SMTP server and port to use, use server:port")
                            .validator(|server_and_port| {
                                let parts: Vec<&str> = server_and_port.split(':').collect();
                                if parts.len() != 2 {
                                    return Err("Not valid server".to_owned());
                                }
                                if parts[1].parse::<u16>().is_err() {
                                    return Err("Port is not in the correct format.".to_owned());
                                }
                                Ok(())
                            }),
                    )
                    .arg(
                        Arg::with_name("upload")
                            .long("upload")
                            .takes_value(true)
                            .index(4)
                            .required(true)
                            .help("Expected upload bandwidth, in mbps (e.g. 123.45)")
                            .validator(|v| {
                                if v.parse::<f64>().is_err() {
                                    return Err("Upload bandwidth is not in the correct format.".to_owned());
                                }
                                Ok(())
                            }),
                    )
                    .arg(
                        Arg::with_name("download")
                            .long("download")
                            .takes_value(true)
                            .index(5)
                            .required(true)
                            .help("Expected download bandwidth, in mbps (e.g. 123.45)")
                            .validator(|v| {
                                if v.parse::<f64>().is_err() {
                                    return Err("Download bandwidth is not in the correct format.".to_owned());
                                }
                                Ok(())
                            }),
                    )
                    .arg(
                        Arg::with_name("simulate")
                            .short("s")
                            .long("simulate")
                            .help("Should write email to stdout instead of sending e-mail"),
                    )
                    .arg(
                        Arg::with_name("threshold")
                            .short("t")
                            .long("threshold")
                            .takes_value(true)
                            .required(true)
                            .help("Threshold percentage. If measured values follow bellow this amount an e-mail message is sent. It has to be an integer.")
                            .default_value("20")
                            .validator(|v| {
                                if v.parse::<u8>().is_err() {
                                    return Err("Threshold is not in the correct format.".to_owned());
                                }
                                Ok(())
                            }),
                    )
                    .arg(
                        Arg::with_name("count")
                            .short("c")
                            .long("count")
                            .takes_value(true)
                            .required(true)
                            .help("How many measurements are used to make up the average")
                            .default_value("8")
                            .validator(|v| {
                                if v.parse::<u8>().is_err() {
                                    return Err("Measurement count is not in the correct format.".to_owned());
                                }
                                Ok(())
                            }),
                    )
                    .arg(
                        Arg::with_name("username")
                            .short("u")
                            .long("username")
                            .takes_value(true)
                            .requires("password")
                            .help("SMTP server user for authentication"),
                    )
                    .arg(
                        Arg::with_name("password")
                            .short("p")
                            .long("password")
                            .takes_value(true)
                            .requires("username")
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
                    )
                    .arg(
                        Arg::with_name("show_results")
                            .long("show-results")
                            .help("Sends results to stdout, one result per line: download, upload, ping"),
                    )
                    .arg(
                        Arg::with_name("sender email")
                            .short("e")
                            .long("sender")
                            .takes_value(true)
                            .requires_all(&["email", "smtp server"])
                            .help("E-mail address to send the alert message from"),
                    )
                    .arg(
                        Arg::with_name("email")
                            .long("to")
                            .takes_value(true)
                            .requires_all(&["sender email", "smtp server"])
                            .help("E-mail address to send the alert message to"),
                    )
                    .arg(
                        Arg::with_name("smtp server")
                            .long("smtp")
                            .takes_value(true)
                            .requires_all(&["sender email", "email"])
                            .help("SMTP server and port to use, use server:port")
                            .validator(|server_and_port| {
                                let parts: Vec<&str> = server_and_port.split(':').collect();
                                if parts.len() != 2 {
                                    return Err("Not valid server".to_owned());
                                }
                                if parts[1].parse::<u16>().is_err() {
                                    return Err("Port is not in the correct format.".to_owned());
                                }
                                Ok(())
                            }),
                    )
                    .arg(
                        Arg::with_name("username")
                            .short("u")
                            .long("username")
                            .requires("password")
                            .takes_value(true)
                            .help("SMTP server user for authentication"),
                    )
                    .arg(
                        Arg::with_name("password")
                            .short("p")
                            .long("password")
                            .requires("username")
                            .takes_value(true)
                            .help("SMTP server password for authentication"),
                    ),
            )
    }

    fn get_smtp_from_cl(args: &clap::ArgMatches) -> Option<Smtp> {
        let smtp = if let Some(server_and_port) = args.value_of("smtp server") {
            let parts: Vec<&str> = server_and_port.split(':').collect();
            let server = parts[0];
            let port = parts[1].parse::<u16>().ok()?;
            let credentials = if let (Some(username), Some(password)) =
                (args.value_of("username"), args.value_of("password"))
            {
                Some(Credentials {
                    username: username.to_owned(),
                    password: password.to_owned(),
                })
            } else {
                None
            };
            let email = args.value_of("sender email")?;
            Some(Smtp {
                email: email.to_owned(),
                server: server.to_owned(),
                port,
                credentials,
            })
        } else {
            None
        };
        smtp
    }

    fn get_config_from_cl(args: clap::ArgMatches) -> Option<Command> {
        match args.subcommand() {
            ("run", Some(run_args)) => Some(Command::Run(Run {
                simulate: run_args.is_present("simulate"),
                email_options: EmailOptions::new_from_args(run_args),
                show_results: run_args.is_present("show_results"),
            })),
            ("alert", Some(alert_args)) => Some(Command::Alert(Alert {
                simulate: alert_args.is_present("simulate"),
                email: alert_args.value_of("email").unwrap().to_owned(),
                expected_download: alert_args
                    .value_of("download")
                    .unwrap()
                    .parse::<f64>()
                    .unwrap(),
                expected_upload: alert_args
                    .value_of("upload")
                    .unwrap()
                    .parse::<f64>()
                    .unwrap(),
                threshold: alert_args
                    .value_of("threshold")
                    .unwrap()
                    .parse::<u8>()
                    .unwrap(),
                count: alert_args.value_of("count").unwrap().parse::<u8>().unwrap(),
                smtp: Args::get_smtp_from_cl(alert_args).unwrap(),
            })),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Run {
    pub simulate: bool,
    pub email_options: Option<EmailOptions>,
    pub show_results: bool,
}

#[derive(Debug)]
pub struct Alert {
    pub simulate: bool,
    pub email: String,
    pub expected_download: f64,
    pub expected_upload: f64,
    pub threshold: u8,
    pub count: u8,
    pub smtp: Smtp,
}

#[derive(Debug)]
pub struct Smtp {
    pub server: String,
    pub email: String,
    pub port: u16,
    pub credentials: Option<Credentials>,
}

#[derive(Debug)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct EmailOptions {
    pub email: String,
    pub smtp: Smtp,
}
impl EmailOptions {
    pub fn new_from_args(args: &clap::ArgMatches) -> Option<EmailOptions> {
        if let (Some(email), Some(smtp)) = (
            args.value_of("email").map(|str| str.to_owned()),
            Args::get_smtp_from_cl(args),
        ) {
            Some(EmailOptions { email, smtp })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_run_simulated() {
        let run = match Args::new_from(["trackspeedtest", "run", "--simulate"].iter())
            .unwrap()
            .command
            .unwrap()
        {
            Command::Alert(_) => panic!("Should not be alert"),
            Command::Run(run) => run,
        };
        assert!(run.simulate);
    }
}
