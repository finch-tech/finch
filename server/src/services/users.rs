use data_encoding::BASE64;
use futures::future::{Future, IntoFuture};
use ring::rand::SecureRandom;
use ring::{digest, pbkdf2, rand};
use uuid::Uuid;

use auth::{AuthUser, JWTPayload, LoginPayload};
use db::postgres::PgExecutorAddr;
use models::user::{User, UserPayload};
use services::Error;
use types::PrivateKey;

const CREDENTIAL_LEN: usize = digest::SHA512_OUTPUT_LEN;
const N_ITER: u32 = 100_000;

pub fn register(
    mut payload: UserPayload,
    postgres: PgExecutorAddr,
) -> impl Future<Item = User, Error = Error> {
    let rng = rand::SystemRandom::new();
    let mut salt = [0u8; CREDENTIAL_LEN];

    rng.fill(&mut salt).unwrap();

    let mut pbkdf2_hash = [0u8; CREDENTIAL_LEN];
    pbkdf2::derive(
        &digest::SHA512,
        N_ITER,
        &salt,
        payload.password.as_bytes(),
        &mut pbkdf2_hash,
    );

    payload.password = BASE64.encode(&pbkdf2_hash);
    payload.salt = Some(BASE64.encode(&salt));

    User::insert(payload, postgres).from_err()
}

pub fn authenticate(
    payload: LoginPayload,
    postgres: PgExecutorAddr,
    jwt_private: PrivateKey,
) -> impl Future<Item = String, Error = Error> {
    User::find_by_email(payload.email.clone(), postgres)
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
                        payload.password.as_bytes(),
                        &password_hash,
                    ).map_err(|_| Error::IncorrectPassword)
                        .into_future()
                })
                .and_then(move |_| {
                    JWTPayload::new(Some(AuthUser { id: user.id }), None)
                        .encode(&jwt_private)
                        .map_err(|e| Error::from(e))
                })
        })
}

pub fn get(id: Uuid, postgres: PgExecutorAddr) -> impl Future<Item = User, Error = Error> {
    User::find_by_id(id, postgres).from_err()
}
