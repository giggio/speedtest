use std::str;

use lettre::{
    smtp::authentication::Credentials, ClientSecurity, ClientTlsParameters, SmtpClient,
    SmtpTransport, Transport,
};
use lettre_email::Email;

use crate::args::Smtp;

fn get_mailer(smtp: &Smtp) -> Result<SmtpTransport, String> {
    let smtp_client = if let Some(credentials) = &smtp.credentials {
        let mut tls_builder = native_tls::TlsConnector::builder();
        tls_builder.min_protocol_version(Some(lettre::smtp::client::net::DEFAULT_TLS_PROTOCOLS[0]));
        let tls_parameters =
            ClientTlsParameters::new(smtp.server.clone(), tls_builder.build().unwrap());
        SmtpClient::new(
            (smtp.server.clone(), smtp.port),
            ClientSecurity::Wrapper(tls_parameters),
        )
        .map_err(|err| format!("Error when creating smtp client: {}", err))?
        .credentials(Credentials::new(
            credentials.username.clone(),
            credentials.password.clone(),
        ))
    } else {
        SmtpClient::new(&smtp.server, ClientSecurity::None)
            .map_err(|err| format!("Error when creating insecure smtp client: {}", err))?
    };
    Ok(smtp_client.transport())
}

pub fn send_mail(
    simulate: bool,
    email_address: String,
    subject: &str,
    message_body: &str,
    smtp: Smtp,
) -> Result<(), String> {
    if simulate {
        println!(
            "--------------\nWould be sending e-mail message to: {}\nSubject: {}\nBody:\n{}\n--------------\n",
            email_address, subject, message_body
        );
    } else {
        printlnv!("Preparing e-mail...");
        let email = Email::builder()
            .to(email_address.clone())
            .from(smtp.email.clone())
            .subject(subject)
            .text(message_body)
            .build()
            .map_err(|err| format!("Error when creating email: {}", err))?;
        printlnv!("Preparing mailer...");
        let mut mailer = get_mailer(&smtp)?;
        printlnv!(
            "Sending e-mail message to: {}\nSubject: {}\nBody:\n{}",
            email_address,
            subject,
            message_body
        );
        let result = mailer.send(email.into());
        if let Err(err) = result {
            printlnv!("E-mail message was NOT sent successfully.\nError:\n{}", err);
            return Err("Could not send email.".to_owned());
        } else {
            printlnv!("E-mail message was sent successfully.");
        }
    }
    Ok(())
}
