#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use rocket::fairing::AdHoc;
use rocket::fs::{relative, NamedFile};
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar};
use rocket::outcome::IntoOutcome;
use rocket::response::status;
use rocket::response::status::NotFound;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::Json;
use rocket::State;
use rocket::{Build, Rocket};
use rocket_dyn_templates::{Template};
use std::path::{Path, PathBuf};

use lettre::Message;
use rusoto_core::Region;
use rusoto_ses::{RawMessage, SendRawEmailRequest, Ses, SesClient};
use std::env;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use uuid::Uuid;

use chrono::Utc;
use diesel::prelude::*;

pub mod base;
use base::*;

pub mod model;
use model::User;
pub mod schema;

pub mod docs;

#[get("/<path..>", rank=3)]
async fn static_files(path: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new(relative!("site")).join(path);
    NamedFile::open(path)
        .await
        .map_err(|e| NotFound(e.to_string()))
}


#[get("/", rank=2)]
pub fn index() -> Template {
    let ctx = IndexContext { error: "" };
    Template::render("index", &ctx)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserId {
    type Error = std::convert::Infallible;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<UserId, Self::Error> {
        request.cookies()
            .get_private("id")
            .map(|s| Uuid::parse_str(s.value()).map(UserId).ok())
            .flatten()
            .or_forward(())
    }
}

#[get("/")]
pub fn index_user(userid: UserId) -> Template {
    let ctx = UserContext { user_id: &userid.0 };
    Template::render("home", &ctx)
}


#[post("/loginEmail", data = "<email>")]
async fn send_login_email(
    email: Json<LoginEmail<'_>>,
    config: &State<Config>,
    tokens: &State<EmailTokens>,
) -> Result<status::Accepted<Json<String>>, status::Custom<String>> {
    let client = SesClient::new(Region::EuWest3);

    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    tokens
        .tokens
        .lock()
        .unwrap()
        .insert(token.clone(), LoginRegistration::new(email.address));

    let link = if config.port == 443 {
        format!(
            "https://{}.dataregi.com/loginToken?token={}",
            config.callback_name, token
        )
    } else {
        format!(
            "https://{}.dataregi.com:{}/loginToken?token={}",
            config.callback_name, config.port, token
        )
    };

    send_email_ses(
        &client,
        "login@dataregi.com",
        email.address,
        "Login to DataRegi",
        format!(
            "Click on this link to log in to DataRegi: \n{}\nThank you!\n\nDataRegi",
            link
        ),
    )
    .await
    .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

    Ok(status::Accepted(Some(Json(String::from("Email sent")))))
}

#[get("/loginToken?<token>")]
async fn login_from_token(
    token: &str,
    config: &State<Config>,
    tokens: &State<EmailTokens>,
    cookies: &CookieJar<'_>,
    conn: MainDbConn,
) -> Template {
    let maybe_reg = tokens.tokens.lock().unwrap().remove(token);

    let mut error = String::new();
    if let Some(reg) = maybe_reg {
        if reg.timestamp.elapsed().as_secs() > config.token_lifespan_minutes * 60 {
            error = String::from("Could not log in, expired token");
        } else {
            use schema::users::dsl::*;
            let addr = reg.address.clone();
            let rouser = conn
                .run(move |c| users.filter(email.eq(addr)).first::<User>(c).optional())
                .await;

            match rouser {
                Ok(ouser) => {
                    let ruuid = match ouser {
                        None => {
                            let user = User::new_login(reg.address);
                            conn.run(move |c| {
                                diesel::insert_into(users)
                                    .values(&user)
                                    .execute(c)
                                    .map(|_| user.id)
                            })
                            .await
                        }
                        Some(user) => {
                            conn.run(move |c| {
                                diesel::update(&user)
                                    .set(last_login.eq(Utc::now()))
                                    .execute(c)
                                    .map(|_| user.id)
                            })
                            .await
                        }
                    };
                    match ruuid {
                        Ok(uuid) => {
                            let mut c = Cookie::new("id", uuid.to_string());
                            c.set_secure(Some(true));
                            cookies.add_private(c);
                        }
                        Err(e) => {
                            error = e.to_string();
                        }
                    }
                }
                Err(e) => {
                    error = e.to_string();
                }
            }
        }
    } else {
        error = String::from("Could not log in, invalid token");
    }
    let ctx = IndexContext { error: &error };
    if error.is_empty() {
        Template::render("home", &ctx)
    } else {
        Template::render("index", &ctx)
    }
}

async fn send_email_ses(
    ses_client: &SesClient,
    from: &str,
    to: &str,
    subject: &str,
    body: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let email = Message::builder()
        .from(from.parse()?)
        .to(to.parse()?)
        .subject(subject)
        .body(body.to_string())?;

    let raw_email = email.formatted();

    let ses_request = SendRawEmailRequest {
        raw_message: RawMessage {
            data: base64::encode(raw_email).into(),
        },
        ..Default::default()
    };

    ses_client.send_raw_email(ses_request).await?;

    Ok(())
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    embed_migrations!("migrations");

    let conn = MainDbConn::get_one(&rocket)
        .await
        .expect("database connection");
    conn.run(|c| embedded_migrations::run(c))
        .await
        .expect("diesel migrations");

    rocket
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(MainDbConn::fairing())
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
        .mount(
            "/",
            routes![index_user, index, static_files, send_login_email, login_from_token],
        )
        .attach(AdHoc::config::<Config>())
        .manage(EmailTokens::default())
        .attach(Template::fairing())
}
