/// Begins with an underscore because this must be the first file to be called

mod utils;

use std::env;
use crunchyroll_rs::Crunchyroll;

#[tokio::test]
async fn login_with_credentials() {
    let user = env::var("USER").expect("'USER' environment variable not found");
    let password = env::var("PASSWORD").expect("'PASSWORD' environment variable not found");

    let crunchy = Crunchyroll::new()
        .login_with_credentials(user, password)
        .await;

    assert!(crunchy.is_ok(), "{}", crunchy.unwrap_err().to_string());

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).unwrap()
    }
}

#[tokio::test]
async fn login_with_access_token() {
    let access_token = env::var("ACCESS_TOKEN").expect("'ACCESS_TOKEN' environment variable not found");

    let crunchy = Crunchyroll::new()
        .login_with_access_token(access_token)
        .await;

    assert!(crunchy.is_ok(), "{}", crunchy.unwrap_err().to_string());

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).unwrap()
    }
}

#[tokio::test]
async fn login_with_refresh_token() {
    let refresh_token = env::var("REFRESH_TOKEN").expect("'REFRESH_TOKEN' environment variable not found");

    let crunchy = Crunchyroll::new()
        .login_with_refresh_token(refresh_token)
        .await;

    assert!(crunchy.is_ok(), "{}", crunchy.unwrap_err().to_string());

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).unwrap()
    }
}

#[tokio::test]
async fn login_with_etp_rt() {
    let etp_rt = env::var("ETP_RT").expect("'ETP_RT' environment variable not found");

    let crunchy = Crunchyroll::new()
        .login_with_etp_rt(etp_rt)
        .await;

    assert!(crunchy.is_ok(), "{}", crunchy.unwrap_err().to_string());

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).unwrap()
    }
}

#[tokio::test]
async fn login_with_session_id() {
    let session_id = env::var("SESSION_ID").expect("'SESSION_ID' environment variable not found");

    let crunchy = Crunchyroll::new()
        .login_with_session_id(session_id)
        .await;

    assert!(crunchy.is_ok(), "{}", crunchy.unwrap_err().to_string());

    if !utils::session::has_session() {
        utils::session::set_session(crunchy.unwrap()).unwrap()
    }
}
