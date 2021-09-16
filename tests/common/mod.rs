use dataregi;
use rocket::local::blocking::{Client,LocalRequest};
use std::env;
use rocket::http::Cookie;

pub fn setup() -> Client{
    env::set_var("ROCKET_PROFILE","test");

    let rocket = dataregi::rocket();
    let client = Client::tracked(rocket).unwrap();

    client
}

pub fn with_test_login(req: LocalRequest) -> LocalRequest {
    req.private_cookie(Cookie::new("id", "b9518d55-3256-4b96-81d0-65b1d7c4fb38"))
}