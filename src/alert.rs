use chrono::{DateTime, NaiveDateTime, Utc};
use lettre::{
    smtp::authentication::Credentials, ClientSecurity, ClientTlsParameters, SmtpClient,
    SmtpTransport, Transport,
};
use lettre_email::Email;
use rev_lines::RevLines;
use serde::{de, Deserialize, Deserializer};
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

use crate::args::Alert;

pub fn alert(alert: Alert) -> Result<(), Option<String>> {
    let results = match get_latest_results(alert.count)? {
        Some(results) => results,
        None => {
            println!("Not enough results to report yet.");
            return Ok(());
        }
    };
    let average = get_average(results);
    if average_is_bellow(&average, &alert) {
        send_email(average, alert)?;
    }
    Ok(())
}

fn send_email(average: Average, alert: Alert) -> Result<(), String> {
    const SUBJECT: &str = "Bandwith bellow expectation";
    let message_body = format!(
        "Latest bandwidth measurements found a discrepancy.\n\
    Expected badwidth was {} mpbs for download and {} mbps for upload.\n\
    Found {:.2} mbps for download and {:.2} mbps for upload, for the last ~{} hours ({} samples).",
        alert.expected_download,
        alert.expected_upload,
        average.download,
        average.upload,
        average.period_in_hours,
        alert.count
    );
    if alert.simulate {
        println!(
            "Would be sending e-mail message to: {}\nSubject: {}\nBody:\n{}",
            alert.email, SUBJECT, message_body
        );
    } else {
        let email = Email::builder()
            .to(alert.email.clone())
            .from(alert.smtp.email.clone())
            .subject(SUBJECT)
            .text(&message_body)
            .build()
            .map_err(|err| format!("Error when creating email: {}", err))?;
        let mut mailer = get_mailer(&alert)?;
        let result = mailer.send(email.into());
        printlnv!(
            "Sent e-mail message to: {}\nSubject: {}\nBody:\n{}",
            alert.email,
            SUBJECT,
            message_body
        );
        if result.is_ok() {
            printlnv!("E-mail message was sent successfully.");
        } else {
            printlnv!("E-mail message was NOT sent successfully.");
            return Err("Could not send email.".to_owned());
        }
    }
    Ok(())
}

fn get_mailer(alert: &Alert) -> Result<SmtpTransport, String> {
    let smtp_client = if let Some(credentials) = &alert.smtp.credentials {
        let mut tls_builder = native_tls::TlsConnector::builder();
        tls_builder.min_protocol_version(Some(lettre::smtp::client::net::DEFAULT_TLS_PROTOCOLS[0]));
        let tls_parameters =
            ClientTlsParameters::new(alert.smtp.server.clone(), tls_builder.build().unwrap());
        SmtpClient::new(
            (alert.smtp.server.clone(), alert.smtp.port),
            ClientSecurity::Wrapper(tls_parameters),
        )
        .map_err(|err| format!("Error when creating smtp client: {}", err))?
        .credentials(Credentials::new(
            credentials.username.clone(),
            credentials.password.clone(),
        ))
    } else {
        SmtpClient::new(&alert.smtp.server, ClientSecurity::None)
            .map_err(|err| format!("Error when creating insecure smtp client: {}", err))?
    };
    Ok(smtp_client.transport())
}

fn average_is_bellow(average: &Average, alert: &Alert) -> bool {
    average.upload < alert.expected_upload * (1.0 - alert.threshold as f64 / 100.0)
        || average.download < alert.expected_download * (1.0 - alert.threshold as f64 / 100.0)
}

fn get_average(results: Vec<ResultCsv>) -> Average {
    let mut dl = 0.0;
    let mut ul = 0.0;
    let len = results.len();
    let mut min_date = chrono::MAX_DATETIME;
    let mut max_date = chrono::MIN_DATETIME;
    for result in results.into_iter() {
        dl += result.speeds_download;
        ul += result.speeds_upload;
        if result.date < min_date {
            min_date = result.date;
        }
        if result.date > max_date {
            max_date = result.date;
        }
    }
    Average {
        download: dl / len as f64,
        upload: ul / len as f64,
        period_in_hours: ((max_date - min_date).num_minutes() as f64 / 60.0).round() as i64,
    }
}

fn get_latest_results(count: u8) -> Result<Option<Vec<ResultCsv>>, String> {
    let cwd = std::env::current_dir()
        .map_err(|err| format!("Error when finding current working directory: {}", err))?;
    let data_dir = cwd.join("data");
    let file_path = data_dir.join("speed.csv");
    let file = if file_path.exists() {
        File::open(&file_path).map_err(|err| format!("Error when opening summary file: {}", err))?
    } else {
        return Ok(None);
    };
    let mut lines = BufReader::new(&file).lines();
    let first_line = lines.next();
    if lines.count() < count as usize {
        return Ok(None);
    }
    let revlines = RevLines::new(BufReader::new(&file))
        .map_err(|err| format!("Error when opening file: {}", err))?;
    let mut last_lines: Vec<String> = revlines.take(count as usize).collect();
    last_lines.splice(0..0, vec![first_line.unwrap().unwrap()]);
    let text = last_lines.into_iter().fold(String::new(), |mut str, item| {
        str.push_str(&item);
        str.push_str("\n");
        str
    });
    let mut rdr = csv::Reader::from_reader(text.as_bytes());
    let results: Vec<ResultCsv> = rdr
        .deserialize::<ResultCsv>()
        .filter_map(|result| result.ok())
        .collect();
    if results.iter().len() != count as usize {
        return Err("Error deserializing csv.".to_owned());
    }
    Ok(Some(results))
}

fn date_time_from_str<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let ndt = NaiveDateTime::parse_from_str(&s, "%Y/%m/%d %H:%M:%S").map_err(de::Error::custom)?;
    Ok(DateTime::<Utc>::from_utc(ndt, Utc))
}

#[derive(Debug, Deserialize)]
// #[serde(rename_all = "PascalCase")]
#[serde()]
struct ResultCsv {
    #[serde(deserialize_with = "date_time_from_str")]
    date: DateTime<Utc>,
    speeds_download: f64,
    speeds_upload: f64,
}

#[derive(PartialEq, Debug)]
struct Average {
    upload: f64,
    download: f64,
    period_in_hours: i64,
}

#[cfg(test)]
mod tests {
    mod calculate_average {
        use chrono::prelude::*;
        use chrono::Utc;
        use pretty_assertions::assert_eq;

        use super::super::*;
        #[test]
        fn average_calculated_with_single_item() {
            assert_eq!(
                Average {
                    download: 100.0,
                    upload: 200.0,
                    period_in_hours: 0
                },
                get_average(vec![ResultCsv {
                    date: Utc::now(),
                    speeds_download: 100.0,
                    speeds_upload: 200.0,
                }])
            );
        }

        #[test]
        fn average_calculated_with_two_items() {
            assert_eq!(
                Average {
                    download: 60.0,
                    upload: 120.0,
                    period_in_hours: 2
                },
                get_average(vec![
                    ResultCsv {
                        date: Utc.ymd(2021, 1, 1).and_hms(0, 0, 0),
                        speeds_download: 20.0,
                        speeds_upload: 40.0,
                    },
                    ResultCsv {
                        date: Utc.ymd(2021, 1, 1).and_hms(2, 0, 0),
                        speeds_download: 100.0,
                        speeds_upload: 200.0,
                    }
                ])
            );
        }

        #[test]
        fn average_approximate_hours() {
            assert_eq!(
                Average {
                    download: 1.0,
                    upload: 1.0,
                    period_in_hours: 2
                },
                get_average(vec![
                    ResultCsv {
                        date: Utc.ymd(2021, 1, 1).and_hms(0, 0, 0),
                        speeds_download: 1.0,
                        speeds_upload: 1.0,
                    },
                    ResultCsv {
                        date: Utc.ymd(2021, 1, 1).and_hms(1, 59, 0),
                        speeds_download: 1.0,
                        speeds_upload: 1.0,
                    }
                ])
            );
        }
    }

    mod check_average {
        use crate::args::Smtp;

        use super::super::*;
        #[test]
        fn when_has_one_value_exactly_at_the_average_it_is_ok() {
            assert!(!average_is_bellow(
                &Average {
                    download: 100.0,
                    upload: 100.0,
                    period_in_hours: 5
                },
                &create_alert(0, 100.0, 100.0)
            ));
        }

        #[test]
        fn when_has_download_bellow_average_it_is_not_ok() {
            assert!(average_is_bellow(
                &Average {
                    download: 10.0,
                    upload: 100.0,
                    period_in_hours: 5
                },
                &create_alert(0, 100.0, 100.0)
            ));
        }

        #[test]
        fn when_has_upload_bellow_average_it_is_not_ok() {
            assert!(average_is_bellow(
                &Average {
                    download: 100.0,
                    upload: 10.0,
                    period_in_hours: 5
                },
                &create_alert(0, 100.0, 100.0)
            ));
        }

        #[test]
        fn when_has_one_value_bellow_average_but_within_threshold_it_is_ok() {
            assert!(!average_is_bellow(
                &Average {
                    download: 90.0,
                    upload: 90.0,
                    period_in_hours: 5
                },
                &create_alert(20, 100.0, 100.0)
            ));
        }

        fn create_alert(threshold: u8, download: f64, upload: f64) -> Alert {
            Alert {
                simulate: false,
                count: 1,
                threshold: threshold,
                expected_download: download,
                expected_upload: upload,
                email: "".to_owned(),
                smtp: Smtp {
                    email: "".to_owned(),
                    server: "".to_owned(),
                    port: 0,
                    credentials: None,
                },
            }
        }
    }
}
