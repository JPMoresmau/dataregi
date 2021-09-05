
#[macro_use] extern crate rocket;

use rocket::State;
use rocket::serde::{Deserialize, Serialize, json::Json};
use rocket::response::status::{NotFound};
use rocket::fs::{NamedFile,relative};
use std::path::{Path,PathBuf};
use rocket::response::status;
use rocket::http::Status;
use rocket::fairing::AdHoc;
use rocket_dyn_templates::{Template};
use rocket::http::{Cookie, CookieJar, SameSite};

use lettre::Message;
use rusoto_core::Region;
use rusoto_ses::{RawMessage, SendRawEmailRequest, Ses, SesClient};
use std::env;
use  std::sync::Mutex;
use std::collections::HashMap;

use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use std::time::{Instant};

#[get("/<path..>")]
async fn static_files(path: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new(relative!("site")).join(path);
    NamedFile::open(path).await.map_err(|e| NotFound(e.to_string()))
}


#[derive(Serialize)]
struct IndexContext<'r>{
    error: &'r str, 
}

#[get("/")]
fn index() -> Template {
    let ctx = IndexContext{
        error: ""
    };
    Template::render("index", &ctx)
}

#[derive(Deserialize)]
struct LoginEmail<'r> {
    address: &'r str,
}

#[derive(Deserialize)]
struct Config {
    port: u16,
    callback_name: String,
    token_lifespan_minutes: u64,
}


struct LoginRegistration {
    address: String,
    timestamp: Instant,
}

impl LoginRegistration {
    fn new<S: Into<String>>(address: S) -> Self {
        LoginRegistration{address: address.into(),timestamp: Instant::now()}
    }
}


struct EmailTokens {
    tokens: Mutex<HashMap<String,LoginRegistration>>,
}

impl Default for EmailTokens {
    fn default() -> Self {
        EmailTokens{tokens:Mutex::new(HashMap::new())}
    }
}


#[post("/loginEmail", data="<email>")]
async fn send_login_email(email: Json<LoginEmail<'_>>, config: &State<Config>, tokens: &State<EmailTokens>) -> Result<status::Accepted<Json<String>>,status::Custom<String>>{
    let client=SesClient::new(Region::EuWest3);

    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    tokens.tokens.lock().unwrap().insert(token.clone(), LoginRegistration::new(email.address));

    let link=if config.port==443 {
        format!("https://{}.dataregi.com/loginToken?token={}", config.callback_name, token)
    } else {
        format!("https://{}.dataregi.com:{}/loginToken?token={}", config.callback_name, config.port, token)
    };

    send_email_ses(&client,"login@dataregi.com",email.address,
        "Login to DataRegi",format!("Click on this link to log in to DataRegi: \n{}\nThank you!\n\nDataRegi",link))
        .await.map_err(|e| status::Custom(Status::InternalServerError,e.to_string()))?;

    Ok(status::Accepted(Some(Json(String::from("Email sent")))))
}

#[get("/loginToken?<token>")]
async fn login_from_token(token: &str, config: &State<Config>, tokens: &State<EmailTokens>,cookies: &CookieJar<'_>) -> Template {
    let maybe_reg=tokens.tokens.lock().unwrap().remove(token);

    let mut error = "";
    if let Some(reg) = maybe_reg {
        if reg.timestamp.elapsed().as_secs()>config.token_lifespan_minutes*60 {
            error = "Could not log in, expired token";
        } else {
            let mut c = Cookie::new("email",reg.address);
            c.set_secure(Some(true));
            c.set_http_only(Some(true));
            c.set_same_site(SameSite::Lax);
            cookies.add_private(c);
        }
    } else {
        error = "Could not log in, invalid token";
    }
    let ctx = IndexContext{
        error: error
    };
    if error.is_empty(){
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


#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, static_files, send_login_email, login_from_token])
        .attach(AdHoc::config::<Config>())
        .manage(EmailTokens::default())
        .attach(Template::fairing())

}


