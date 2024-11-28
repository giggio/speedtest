use crate::args::EmailOptions;
use crate::args::Run;
use crate::mail;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::Stdio;

pub fn run(run: Run) -> Result<(), Option<String>> {
    let json_result = run_speedtest(run.simulate, run.email_options)?;
    let result = convert_json(json_result)?;
    write_to_result_file(&result)?;
    append_to_summary_file(&result)?;
    if run.show_results {
        println!("{}", &result.download);
        println!("{}", &result.upload);
        println!("{}", &result.ping);
    }
    printlnv!("Got results:\n{:?}", &result);
    Ok(())
}

fn write_to_result_file(result: &SpeedResult) -> Result<(), String> {
    let cwd = env::current_dir()
        .map_err(|err| format!("Error when finding current working directory: {}", err))?;
    let data_dir = cwd.join("data");
    if !data_dir.exists() {
        std::fs::create_dir(&data_dir)
            .map_err(|err| format!("Error when creating data directory: {}", err))?;
    }
    let file_name = format!("{}.json", result.date.format("%Y%m%d%H%M%S"));
    let file_path = data_dir.join(file_name);
    fs::write(file_path, result.jsonresult.as_bytes())
        .map_err(|err| format!("Error when writing to file: {}", err))?;
    Ok(())
}

fn append_to_summary_file(result: &SpeedResult) -> Result<(), String> {
    let cwd = env::current_dir()
        .map_err(|err| format!("Error when finding current working directory: {}", err))?;
    let data_dir = cwd.join("data");
    let file_path = data_dir.join("speed.csv");
    let mut file = if file_path.exists() {
        OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(file_path)
            .map_err(|err| format!("Error when creating file: {}", err))?
    } else {
        let mut file =
            File::create(&file_path).map_err(|err| format!("Error creating file: {}", err))?;
        file.write_all("date,ping,speeds_download,speeds_upload,client_ip,client_isp,server_host,server_lat,server_lon,server_location,server_country,location_distance,server_ping,server_id\n".as_bytes())
            .map_err(|err| format!("Error writing header to file: {}", err))?;
        file
    };
    let line = format!(
        r#"{},{},{:.2},{:.2},"{}","{}","{}",null,null,"{}","{}",null,null,{}{}"#,
        result.date.format("%Y/%m/%d %H:%M:%S"),
        result.ping,
        result.download * 8.0 / 1024.0 / 1024.0,
        result.upload * 8.0 / 1024.0 / 1024.0,
        result.client_ip,
        result.client_isp,
        result.server_host,
        result.server_location,
        result.server_country,
        result.server_id,
        "\n"
    );
    file.write(line.as_bytes())
        .map_err(|err| format!("Error when writing to file: {}", err))?;
    Ok(())
}

fn run_speedtest(simulate: bool, email_options: Option<EmailOptions>) -> Result<String, String> {
    let (speedtestbin, args) = find_speedtest_binary_and_args(simulate)?;
    let child = std::process::Command::new(&speedtestbin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| {
            format!(
                "Could not run {}.\nError:\n{}",
                speedtestbin.to_str().unwrap_or("<filename not found>"),
                err
            )
        })?;
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Could wait for speedtest execution.\nError:\n{}", e))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stdout_text = String::from_utf8_lossy(&output.stdout);
        let mut error_message = if stdout_text.is_empty() {
            format!(
                "Speedtest executable exited with an error and no output. Errors:\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        } else {
            format!(
                "Speedtest executable exited with an error. Output:\n{}\nErrors:\n{}",
                stdout_text,
                String::from_utf8_lossy(&output.stderr)
            )
        };
        if let Err(msg) = send_email_on_error(simulate, &error_message, email_options) {
            error_message += &format!("\nAlso, could not send e-mail. Error:\n{}", &msg);
        };
        Err(error_message)
    }
}

fn find_speedtest_binary_and_args<'a>(simulate: bool) -> Result<(PathBuf, Vec<&'a str>), String> {
    let (bin, args) = if simulate {
        (
            "echo",
            vec![
                r#"{"type":"result","timestamp":"2021-01-03T12:10:00Z","ping":{"jitter":0.28499999999999998,"latency":5.7279999999999998},"download":{"bandwidth":20309419,"bytes":176063552,"elapsed":8815},"upload":{"bandwidth":13206885,"bytes":195610380,"elapsed":15015},"packetLoss":0,"isp":"Some ISP","interface":{"internalIp":"192.168.1.2","name":"eth0","macAddr":"99:99:99:99:99:99","isVpn":false,"externalIp":"84.6.0.1"},"server":{"id":99999,"name":"Some Server","location":"São Paulo","country":"Brazil","host":"someserver.nonexistentxyz.com","port":10000,"ip":"15.22.77.1"},"result":{"id":"babad438-ac4b-47db-bc28-2de7e257bd28","url":"https://www.fakespeedtest.net/result/c/babad438-ac4b-47db-bc28-2de7e257bd28"}}"#,
            ],
        )
    } else {
        (
            "speedtest",
            vec![
                "--accept-license",
                "--accept-gdpr",
                "--format=json",
                "--progress=no",
            ],
        )
    };
    match which::which(bin) {
        Ok(speedtestbin) => Ok((speedtestbin, args)),
        Err(_) => {
            let cwd = env::current_dir()
                .map_err(|err| format!("Error when finding current working directory: {}", err))?;
            let speedtestbin = cwd.join(bin);
            if speedtestbin.exists() {
                Ok((speedtestbin, args))
            } else {
                Err("Could not find speedtest binary.".to_owned())
            }
        }
    }
}

fn convert_json(json: String) -> Result<SpeedResult, String> {
    let result: serde_json::Result<RawSpeedResult> = serde_json::from_str(&json);
    match result {
        Ok(raw_result) => Ok(SpeedResult {
            client_ip: raw_result.interface.external_ip,
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
                String::from_utf8_lossy(json.as_bytes()),
                err
            );
            Err(msg)
        }
    }
}

fn send_email_on_error(
    simulate: bool,
    message_body: &str,
    optinal_email_options: Option<EmailOptions>,
) -> Result<(), String> {
    if let Some(email_options) = optinal_email_options {
        mail::send_mail(
            simulate,
            email_options.email,
            "Could not measure bandwidth",
            message_body,
            email_options.smtp,
        )?;
    }
    Ok(())
}

#[derive(Derivative)]
#[derivative(Debug)]
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
    #[derivative(Debug = "ignore")]
    jsonresult: String,
}

#[derive(Deserialize)]
struct RawSpeedResult {
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
#[serde(rename_all = "camelCase")]
struct RawInterface {
    external_ip: String,
}
#[derive(Deserialize)]
struct RawServer {
    host: String,
    location: String,
    country: String,
    id: u32,
}
