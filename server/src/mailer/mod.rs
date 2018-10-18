mod errors;
pub use self::errors::Error;

use std::time::Duration;

use actix::prelude::*;
use lettre::smtp::authentication::{Credentials, Mechanism};
use lettre::smtp::response::Response;
use lettre::smtp::ConnectionReuseParameters;
use lettre::smtp::SmtpTransportBuilder;
use lettre::{ClientSecurity, ClientTlsParameters, EmailTransport, SmtpTransport};
use lettre_email::EmailBuilder;
use native_tls::Protocol;
use native_tls::TlsConnector;

pub type MailerAddr = Addr<Mailer>;

pub fn init_mailer(
    smtp_host: String,
    smtp_port: u16,
    smtp_user: String,
    smtp_pass: String,
) -> SmtpTransport {
    let mut tls_builder = TlsConnector::builder().unwrap();

    tls_builder
        .supported_protocols(&[Protocol::Tlsv10])
        .unwrap();

    let tls_parameters = ClientTlsParameters::new(smtp_host.clone(), tls_builder.build().unwrap());

    SmtpTransportBuilder::new(
        (&smtp_host[..], smtp_port),
        ClientSecurity::Required(tls_parameters),
    )
    .expect("Failed to create transport")
    .authentication_mechanism(Mechanism::Login)
    .credentials(Credentials::new(smtp_user.clone(), smtp_pass.clone()))
    .connection_reuse(ConnectionReuseParameters::NoReuse)
    .timeout(Some(Duration::new(15, 0)))
    .build()
}

pub struct Mailer(pub SmtpTransport);

impl Actor for Mailer {
    type Context = SyncContext<Self>;

    fn stopped(&mut self, _: &mut Self::Context) {
        self.0.close();
    }
}

#[derive(Message)]
#[rtype(result = "Result<Response, Error>")]
pub struct SendMail {
    pub subject: String,
    pub from: String,
    pub to: String,
    pub html: String,
    pub text: String,
}

impl Handler<SendMail> for Mailer {
    type Result = Result<Response, Error>;

    fn handle(
        &mut self,
        SendMail {
            subject,
            from,
            to,
            html,
            text,
        }: SendMail,
        _: &mut Self::Context,
    ) -> Self::Result {
        let email = EmailBuilder::new()
            .to(&to[..])
            .from((&from[..], ""))
            .subject(&subject[..])
            .alternative(&html[..], &text[..])
            .build()
            .unwrap();

        self.0.send(&email).map_err(|e| Error::from(e))
    }
}
