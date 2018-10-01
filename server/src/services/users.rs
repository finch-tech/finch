use std::env;

use chrono::{prelude::*, Duration};
use data_encoding::BASE64;
use futures::future::{err, Future, IntoFuture};
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use uuid::Uuid;

use auth::{AuthUser, JWTPayload};
use core::db::postgres::PgExecutorAddr;
use core::user::{User, UserPayload};
use mailer::{MailerAddr, SendMail};
use services::Error;
use types::PrivateKey;

const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
const N_ITER: u32 = 100_000;

pub fn register(
    mut payload: UserPayload,
    mailer: MailerAddr,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = User, Error = Error> {
    let postgres = postgres.clone();
    let rng = rand::SystemRandom::new();
    let mut salt = [0u8; CREDENTIAL_LEN];

    let web_client_url =
        env::var("WEB_CLIENT_URL").expect("WEB_CLIENT_URL environment variable must be set.");
    let registration_mail_sender = env::var("REGISTRATION_MAIL_SENDER")
        .expect("REGISTRATION_MAIL_SENDER environment variable must be set.");

    rng.fill(&mut salt).unwrap();

    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        &digest::SHA512,
        N_ITER,
        &salt,
        payload.password.unwrap().as_bytes(),
        &mut pbkdf2_hash,
    );

    payload.password = Some(BASE64.encode(&pbkdf2_hash));
    payload.salt = Some(BASE64.encode(&salt));

    // Delete user with the same email if its verification_token is expired.
    User::delete_expired(payload.email.clone().unwrap(), &postgres)
        .from_err()
        .and_then(move |_| {
            User::insert(payload, &postgres)
                .from_err()
                .and_then(move |user| {
                    let user_id = user.id;

                    let url = format!(
                        "{}/activation?token={}",
                        web_client_url, user.verification_token
                    );

                    let html = format!(
                        "Please click the following link to activate your account: <a href=\"{}\">{}</a>.",
                        url, url
                    );

                    let text = format!(
                        "Please click the following link to activate your account: {}",
                        url
                    );

                    mailer
                        .send(SendMail {
                            subject: String::from("Please activate your account."),
                            from: registration_mail_sender,
                            to: user.email.clone(),
                            html,
                            text,
                        })
                        .from_err()
                        .and_then(move |res| res.map_err(|e| Error::from(e)))
                        .then(move |res| res.and_then(|_| Ok(user)))
                        .from_err()
                        .or_else(move |e| {
                            User::delete(user_id, &postgres)
                                .from_err()
                                .and_then(|_| err(e))
                        })
                })
        })
}

pub fn authenticate(
    email: String,
    password: String,
    postgres: &PgExecutorAddr,
    jwt_private: PrivateKey,
) -> impl Future<Item = (String, User), Error = Error> {
    User::find_by_email(email, postgres)
        .from_err()
        .and_then(move |user| {
            let salt = BASE64
                .decode(&user.salt.as_bytes())
                .map_err(|e| Error::from(e))
                .into_future();

            let password_hash = BASE64
                .decode(&user.password.as_bytes())
                .map_err(|e| Error::from(e))
                .into_future();

            salt.join(password_hash)
                .and_then(move |(salt, password_hash)| {
                    pbkdf2::verify(
                        &digest::SHA512,
                        N_ITER,
                        &salt,
                        password.as_bytes(),
                        &password_hash,
                    ).map_err(|_| Error::IncorrectPassword)
                        .into_future()
                })
                .and_then(move |_| {
                    let expires_at = Utc::now() + Duration::days(1);

                    JWTPayload::new(Some(AuthUser { id: user.id }), None, expires_at)
                        .encode(&jwt_private)
                        .map_err(|e| Error::from(e))
                        .and_then(|token| Ok((token, user)))
                })
        })
}

pub fn activate(
    token: Uuid,
    postgres: &PgExecutorAddr,
    jwt_private: PrivateKey,
) -> impl Future<Item = (String, User), Error = Error> {
    User::activate(token, postgres)
        .from_err()
        .and_then(move |user| {
            let expires_at = Utc::now() + Duration::days(1);

            JWTPayload::new(Some(AuthUser { id: user.id }), None, expires_at)
                .encode(&jwt_private)
                .map_err(|e| Error::from(e))
                .and_then(|token| Ok((token, user)))
        })
}

pub fn reset_password(
    email: String,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = bool, Error = Error> {
    let postgres = postgres.clone();

    User::find_by_email(email, &postgres)
        .from_err()
        .and_then(move |user| {
            let mut payload = UserPayload::from(user.clone());

            payload.set_reset_token();
            User::update(user.id, payload, &postgres)
                .from_err()
                .map(|_| true)
        })
}

pub fn change_password(
    token: Uuid,
    password: String,
    postgres: &PgExecutorAddr,
) -> impl Future<Item = User, Error = Error> {
    let postgres = postgres.clone();

    User::find_by_reset_token(token, &postgres)
        .from_err()
        .and_then(move |user| {
            let mut payload = UserPayload::from(user.clone());

            let rng = rand::SystemRandom::new();
            let mut salt = [0u8; CREDENTIAL_LEN];

            rng.fill(&mut salt).unwrap();

            let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
            pbkdf2::derive(
                &digest::SHA512,
                N_ITER,
                &salt,
                password.as_bytes(),
                &mut pbkdf2_hash,
            );

            payload.password = Some(BASE64.encode(&pbkdf2_hash));
            payload.salt = Some(BASE64.encode(&salt));

            User::update(user.id, payload, &postgres).from_err()
        })
}

pub fn get(id: Uuid, postgres: &PgExecutorAddr) -> impl Future<Item = User, Error = Error> {
    User::find_by_id(id, postgres).from_err()
}
