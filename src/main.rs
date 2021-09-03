
#[macro_use] extern crate rocket;

use rocket::State;
use rocket::serde::{Deserialize, json::Json};
use rocket::response::status::NotFound;
use rocket::fs::{NamedFile,relative};
use std::path::{Path,PathBuf};
use rocket::response::status;
use rocket::http::Status;
use rocket::fairing::AdHoc;

use lettre::Message;
use rusoto_core::Region;
use rusoto_ses::{RawMessage, SendRawEmailRequest, Ses, SesClient};
use std::env;

#[get("/<path..>")]
async fn static_files(path: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let mut path = Path::new(relative!("site")).join(path);
    if path.is_dir() {
        path.push("static/index.html");
    }

    NamedFile::open(path).await.map_err(|e| NotFound(e.to_string()))
}

#[derive(Deserialize)]
struct LoginEmail<'r> {
    address: &'r str,
}

#[derive(Deserialize)]
struct Config {
    port: u16,
    callback_name: String,
}



#[post("/loginEmail", data="<email>")]
async fn send_login_email(email: Json<LoginEmail<'_>>, config: &State<Config>) -> Result<status::Accepted<Json<String>>,status::Custom<String>>{
    let client=SesClient::new(Region::EuWest3);

    let link=format!("https://{}.dataregi.com:{}",config.callback_name,config.port);

    send_email_ses(&client,"login@dataregi.com",email.address,
        "Login to DataRegi",format!("Click on this link to log in to DataRegi: \n{}\nThank you!\n\nDataRegi",link))
        .await.map_err(|e| status::Custom(Status::InternalServerError,e.to_string()))?;

    Ok(status::Accepted(Some(Json(String::from("Email sent")))))
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
        .mount("/", routes![static_files, send_login_email])
        .attach(AdHoc::config::<Config>())

}


