#[macro_use]
extern crate rocket;
#[macro_use]
extern crate diesel_migrations;
#[macro_use]
extern crate diesel;

use diesel::dsl::any;
use orgs::get_organization_by_name;
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::fs::{relative, NamedFile};
use rocket::http::Status;
use rocket::http::{Cookie, CookieJar, SameSite};
use rocket::outcome::IntoOutcome;
use rocket::request::{FromRequest, Request,FlashMessage, Outcome};
use rocket::response::{status, status::NotFound, Flash, Redirect};
use rocket::serde::{Deserialize, Serialize};
use rocket::serde::json::{Value, Json};
use rocket::{State};
use rocket::{Build, Rocket};
use rocket::tokio::{time,io::AsyncReadExt};
use rocket_dyn_templates::Template;
use std::path::{Path, PathBuf};
use std::time::Duration;
use lettre::Message;
use rusoto_ses::{RawMessage, SendRawEmailRequest, Ses, SesClient};
use rusoto_sqs::{Message as SqsMessage, Sqs, SqsClient, ReceiveMessageRequest, DeleteMessageRequest};
use rusoto_s3::{S3,S3Client,GetObjectRequest, DeleteObjectRequest};
use std::{env};

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use chrono::{DateTime,Utc};
use diesel::prelude::*;
use futures::future::join_all;

use jsonwebtoken_google::Parser;
// use email_parser::{email::Email,mime::Entity};
use mailparse::*;
use uuid::Uuid;

pub mod base;
use base::*;

pub mod model;
use model::{User, Member};
pub mod schema;
use schema::limits::user_id;
use schema::limits::dsl::limits as lts;
use schema::members::dsl::members;
use schema::members as mbrs;
use schema::tokens::dsl::tokens;
use schema::tokens as toks;

pub mod accesses;
use crate::accesses::add_access_system;
pub mod docs;
use crate::docs::upload_data;
use crate::model::Token;
pub mod limits;
pub mod orgs;

//const SQS_QUEUE: &str ="https://sqs.eu-west-1.amazonaws.com/334979221948/dataregi-emails-queue";


#[get("/static/<path..>", rank = 3)]
async fn static_files(path: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let path = Path::new(relative!("site")).join(path);
    NamedFile::open(path)
        .await
        .map_err(|e| NotFound(e.to_string()))
}

#[get("/favicon.ico")]
async fn favicon() -> Result<NamedFile, NotFound<String>> {
    static_files(PathBuf::from("favicon.ico")).await

}

fn callback_address(config: &State<Config>) -> String {
     if config.port == 443 {
        format!(
            "{}.dataregi.com",
            config.callback_name
        )
    } else {
        format!(
            "{}.dataregi.com:{}",
            config.callback_name, config.port
        )
    }
}

#[get("/", rank = 2)]
pub fn index(flash: Option<FlashMessage>,config: &State<Config>) -> Template {
    let ctx = IndexContext { error: "", 
        callback_name: &callback_address(config),
        message: &flash.map(|f| f.message().to_string()).unwrap_or_else(String::new) };
    Template::render("index", &ctx)
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserContext {
    type Error = DRError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let ouc=request
            .cookies()
            .get_private(COOKIE)
            .map(|s| serde_json::from_str(s.value()).ok())
            .flatten();

        if request.uri()=="/"{
            ouc.or_forward(())
        } else {
            ouc.into_outcome((Status::Unauthorized,DRError::UnauthorizedError))
        }
            //.or_forward(())
    }
}

#[get("/")]
pub fn index_user(userid: UserContext) -> Template {
    Template::render("home", &userid)
}

#[post("/loginEmail", data = "<email>")]
async fn send_login_email(
    email: Json<LoginEmail<'_>>,
    config: &State<Config>,
    conn: MainDbConn,
) -> Result<status::Accepted<Json<String>>, status::Custom<String>> {
    let client = SesClient::new(config.aws_region.clone());

    let token: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(20)
        .map(char::from)
        .collect();
    let addr=email.address.into();
    let tk=token.clone();
    conn.run(move |c| {
        diesel::insert_into(tokens)
            .values(Token{
                token:tk,
                email: addr,
                created: Utc::now(),
            })
            .execute(c)
    }).await
    .map_err(|e| status::Custom(Status::InternalServerError, e.to_string()))?;

   
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
    println!("Sending link: {}",link);
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

    Ok(status::Accepted(Some(Json(format!("An email has been sent to {}, please click on the link it contains to login!",email.address)))))
}

#[get("/loginToken?<token>")]
async fn login_from_token(
    token: &str,
    config: &State<Config>,
    cookies: &CookieJar<'_>,
    conn: MainDbConn,
) -> Result<Redirect,Template> {
    let tk=token.to_string();
    let maybe_reg = conn.run(move |c| {
        diesel::delete(tokens.filter(toks::token.eq(tk)))
            .returning((toks::email,toks::created)).get_result::<(String,DateTime<Utc>)>(c).optional()
    }).await
    .map_err(|e| Template::render("index", &IndexContext { error: &e.to_string(), message: "",callback_name: &callback_address(config), }))?;
  

    let error= 
        if let Some((address,timestamp)) = maybe_reg {
            if Utc::now().signed_duration_since(timestamp).num_seconds() > (config.token_lifespan_minutes * 60) as i64 {
                Some(String::from("Could not log in, expired token"))
            } else {
                do_login(&address, None, cookies, &conn).await
            }
        } else {
            Some(String::from("Could not log in, invalid token"))
        };
    if let Some(err) = error {
        let ctx = IndexContext { error: &err, message: "",callback_name: &callback_address(config), };
        Err(Template::render("index", &ctx))
    } else {
       Ok(Redirect::to("/"))
    } 
}

async fn do_login(user_email: &str, user_name: Option<&str>,  cookies: &CookieJar<'_>, conn: &MainDbConn ) -> Option<String> {
    use schema::users::dsl::*;
    let addr = String::from(user_email);
    let rouser = conn
        .run(move |c| users.filter(email.eq(addr)).first::<User>(c).optional())
        .await;

    match rouser {
        Ok(ouser) => {
            let rctx = match ouser {
                None => {
                    let user = User::new_login(user_email,user_name);
                    conn.run(move |c| {
                        let ctx = diesel::insert_into(users)
                            .values(&user)
                            .execute(c)
                            .map(|_| UserContext::new(user.id,false));
                        diesel::insert_into(lts)
                            .values(user_id.eq(user.id))
                            .execute(c)?;
                        ctx
                    })
                    .await
                },
                Some(user) => {
                    conn.run(move |c| {

                        let mbrs=members.filter(mbrs::user_id.eq(user.id)).load::<Member>(c)?;

                        diesel::update(&user)
                            .set(last_login.eq(Utc::now()))
                            .execute(c)
                            .map(|_| 
                                UserContext::new_in_org(user.id,&mbrs,user.site_admin)
                                
                            )
                    })
                    .await
                },
            };
            match rctx {
                Ok(ctx) => {
                    let c = Cookie::build(COOKIE, serde_json::to_string(&ctx).unwrap())
                        .secure(true)
                        .same_site(SameSite::Lax) // so it works from email links
                        .finish();
                    cookies.add_private(c);
                }
                Err(e) => {
                    return Some(e.to_string());
                }
            }
        },
        Err(e) => {
            return Some(e.to_string());
        },
    }
    None
}

#[get("/logout")]
async fn logout(cookies: &CookieJar<'_>) -> Flash<Redirect> {
    cookies.remove_private(Cookie::named(COOKIE));
    Flash::success(Redirect::to("/"), "Successfully logged out.")
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

#[get("/document?<id>")]
pub fn single_doc(ctx: UserContext,id: &str) -> Template {
    let ctx = DocumentContext { user_id: &ctx.user_id, doc_id: id };
    Template::render("doc", &ctx)
}

#[get("/org?<id>")]
pub fn single_org(ctx: UserContext,id: &str) -> Template {
    let ctx = OrganizationContext { user_id: &ctx.user_id, org_id: id };
    Template::render("org", &ctx)
}

#[get("/profile")]
pub fn profile(ctx: UserContext) -> Template {
    Template::render("profile", &ctx)
}

#[derive(FromForm,Debug)]
struct GoogleData<'a> {
    g_csrf_token: &'a str,
    credential: &'a str,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub aud: String,         // Optional. Audience
    pub exp: usize,          // Required (validate_exp defaults to true in validation). Expiration time (as UTC timestamp)
    pub iat: usize,          // Optional. Issued at (as UTC timestamp)
    pub iss: String,         // Optional. Issuer
    pub nbf: usize,          // Optional. Not Before (as UTC timestamp)
    pub sub: String,         // Optional. Subject (whom token refers to)
    pub email: String,
    pub name: String,
}

#[post("/google_redirect", data = "<google>")]
async fn google_redirect(google: Form<GoogleData<'_>>, cookies: &CookieJar<'_>, config: &State<Config>,conn: MainDbConn,) -> Result<Redirect,Template> {
    
    let mstr=cookies.get("g_csrf_token").map(|c| c.value());
    //println!("cookie: {:?}",mstr);
    //println!("data: {:?}",google);
    
    let error=
        if mstr==Some(google.g_csrf_token){
            let parser = Parser::new("388424249291-9nu7ati713lrngalv6abai5l5clsatvg.apps.googleusercontent.com");
            let rclaims = parser.parse::<Claims>(&google.credential).await;
            match rclaims {
                Ok(claims)=>{
                    do_login(&claims.email,Some(&claims.name), cookies, &conn).await
                },
                Err(e)=>{
                    Some(e.to_string())
                },
            }
        } else {
            Some(String::from("Error matching token and form"))
        };
    if let Some(err) = error {
        let ctx = IndexContext { error: &err, message: "",callback_name: &callback_address(config), };
        Err(Template::render("index", &ctx))
    } else {
       Ok(Redirect::to("/"))
    } 
   
}

#[catch(401)]
pub fn no_auth() -> Redirect {
    Redirect::to("/")
}

#[catch(401)]
pub fn no_auth_api() -> status::Unauthorized<()> {
    status::Unauthorized(None)
}


async fn sqs_polling<'a>(conn: &'a MainDbConn,config: &Config){
    let sqs = SqsClient::new(config.aws_region.clone());
    let s3 = S3Client::new(config.aws_region.clone());
    let mut go_on=true;
    while go_on {
        go_on = false;
        let rm = ReceiveMessageRequest{
            max_number_of_messages: Some(10),
            queue_url: config.aws_queue.clone(),
            ..Default::default()
        };
        let rrmr=sqs.receive_message(rm).await;
        match rrmr {
            Err(err) => println!("Error: {}",err),
            Ok(rmr) => {
                if let Some(msgs)=rmr.messages {
                    go_on=msgs.len()>0;
                    join_all(msgs.into_iter().map(|msg| process_email(msg,&sqs, &s3,conn,config))).await;
                }
            }
        }
    }
    /*
    let rcnt: Result<i64, Error> = conn.run(|c| 
        users.select(diesel::dsl::count(usrs::name)).get_result(c)
    ).await;
    match rcnt {
        Ok(cnt)=>println!("users: {}",cnt),
        Err(e)=>println!("error! {}",e),
    };*/
    
}

async fn process_email(msg:SqsMessage, sqs:&SqsClient, s3: &S3Client, conn: &MainDbConn,config: &Config) {
    match try_process_email(msg,sqs, s3,conn,config).await {
        Err(e)=> println!("Error processing email: {}",e),
        _=>(),
    };
    
}

async fn try_process_email(msg:SqsMessage, sqs:&SqsClient, s3: &S3Client, conn: &MainDbConn,config: &Config) -> Result<(),Box<dyn std::error::Error>>{

    if let Some(receipt_handle) = msg.receipt_handle{
        println!("Received SQS message: {}",receipt_handle);

        if let Some(body) = msg.body {
            let v:Value=serde_json::from_str(&body)?;
            //println!("Received SNS message: {}",v);
            if let Value::String(s) = &v["Message"]{    
                let msg:Value=serde_json::from_str(s)?;
                //println!("Received SES message: {}",msg["mail"]);
                //println!("Received Receipt: {}",msg["receipt"]);
                let action =&msg["receipt"]["action"];
                if let Value::String(bucket_name)= &action["bucketName"]{
                    if let Value::String(object_key)= &action["objectKey"]{
                        println!("Should download from bucket {}, object {}",bucket_name,object_key);        
                        let gor=s3.get_object(GetObjectRequest{
                            bucket:bucket_name.clone(),
                            key: object_key.clone(),
                            ..Default::default()
                        }).await?;

                        if let Some(body) = gor.body {
                            let mut data =Vec::new();
                            body.into_async_read().read_to_end(&mut data).await?;
                            /*let email = Email::parse(&data)?;
                            println!("From: {}@{}",email.sender.address.local_part,email.sender.address.domain);
                            let rentity=email.mime_entity;
                            let r_entity=rentity.parse()?;
                            if let Entity::Multipart{content,..} = r_entity {
                                for re in content.into_iter(){
                                    let r_entity2=re.parse()?;
                                    if let Entity::Text{subtype,value} = r_entity2 {
                                        println!("MIME entity: {}",subtype);
                                    }
                                }
                            }*/
                            
                            let email = parse_mail(&data)?;
                            let oaddrs= get_uuid_addresses(&email, conn).await?;

                            if let Some(mut addrs) = oaddrs {
                                
                                if !addrs.orgs.is_empty() {
                                    let a=addrs.clone();
                                    let ombr: Option<Member> = conn.run(move |c| {
                                        members.filter(mbrs::user_id.eq(&a.from).and(mbrs::org_id.eq(any(&a.orgs)))).first(c).optional()
                                    }).await?;
                                    if let Some(mbr) = ombr {
                                        addrs.orgs=vec![mbr.org_id];
                                    }
                                }

                                write_attachments(email,addrs,conn).await?;
                            }

                            // delete S3 object
                            s3.delete_object(DeleteObjectRequest {bucket:bucket_name.clone(),
                                key: object_key.clone(),
                                ..Default::default()}).await?;
                        }
                        
                    }
                }
            }
            // delete SQS message
            sqs.delete_message(DeleteMessageRequest{
                    queue_url: config.aws_queue.clone(),
                    receipt_handle: receipt_handle
            }).await?;
        }
    }
    Ok(())

    
}

#[derive(Clone)]
struct Addresses {
    from: Uuid,
    shared: Vec<Uuid>,
    orgs: Vec<Uuid>,
}

async fn get_uuid_addresses<'a>(email: &'a ParsedMail<'a>, conn: &'a MainDbConn) -> Result<Option<Addresses>,Box<dyn std::error::Error>> {
    let mut ofrom= None;
    let mut shared= vec![];
    let mut orgs = vec![];
    for h in email.headers.iter() {
        if "From"==&h.get_key() {
            let list= addrparse_header(&h)?;
            println!("From: {}",list);
            if let Some(fr) = list.extract_single_info() {
                ofrom=Some(ensure_user_exists(&fr.addr,conn).await?);
            } 
        } else if "To"==&h.get_key() || "Cc"==&h.get_key(){
            let list= addrparse_header(&h)?;
            for ma in list.iter(){
                match ma {
                    MailAddr::Single (si) => ensure_real_user_exists(&si.addr,&mut shared, &mut orgs, conn).await?,
                    MailAddr::Group(gi)=> {
                        for si in gi.addrs.iter(){
                            ensure_real_user_exists(&si.addr,&mut shared,&mut orgs,conn).await?;
                        }
                    },
                }
            }
        }
        //println!("{}: {}",h.get_key(),h.get_value());
    }
    Ok(ofrom.map(|from| Addresses{from,shared,orgs}))

} 


async fn ensure_real_user_exists(user_email: &str, shared: &mut Vec<Uuid>, orgs: &mut Vec<Uuid>, conn: &MainDbConn) -> DRResult<()> {
    if let Some(ix) = user_email.find("@dataregi.com"){
        let org_name=&user_email[..ix];
        if let Some(org) = get_organization_by_name(String::from(org_name),conn).await? {
            orgs.push(org.id);
        } else if "register"!=org_name{
            println!("Cannot find organization {}",org_name);
        }
    } else {
        shared.push(ensure_user_exists(user_email,conn).await?);
    } 
    Ok(())
}

async fn write_attachments<'a>(email: ParsedMail<'a>, mut addrs: Addresses, conn: &'a MainDbConn) -> Result<(),Box<dyn std::error::Error>>{
    let org_id=addrs.orgs.pop();
    for p in email.subparts.iter(){
        let cd=p.get_content_disposition();
        if DispositionType::Attachment == cd.disposition{
            if let Some(file_name) = cd.params.get("filename"){
                //println!("{}: {:?}",file_name, p.ctype);
                
                let data=p.get_body_raw()?;
                let du = upload_data(addrs.from,file_name.to_owned(),org_id,Some(p.ctype.mimetype.to_owned()),data,conn).await?;
                if let Some(doc_id) = du.get_id(){
                    println!("Uploaded from email: {}->{}",file_name,doc_id);
                    for uid in addrs.shared.iter(){
                        add_access_system(doc_id,uid.clone(),conn).await?;
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn rocket() -> rocket::Rocket<Build> {
    rocket::build()
        .attach(MainDbConn::fairing())
        .attach(AdHoc::on_ignite("Diesel Migrations", run_migrations))
        .attach(AdHoc::on_liftoff("SQS polling", |rocket| {
            
            Box::pin(async move {
                let conn = MainDbConn::get_one(&rocket)
                        .await
                        .expect("database connection");
                let config: Config = (*rocket.state::<Config>().expect("config")).clone();
                rocket::tokio::spawn(async move {
                    let mut interval = time::interval(Duration::from_secs(10));
                    loop {
                        sqs_polling(&conn, &config).await;
                        interval.tick().await;
                    }
                });
            })
        }))
        .mount(
            "/",
            routes![
                index_user,
                index,
                static_files,
                favicon,
                send_login_email,
                login_from_token,
                logout,
                single_doc,
                profile,
                single_org,
                google_redirect,
            ],
        )
        .mount("/api/docs",docs::routes())
        .mount("/api/accesses",accesses::routes())
        .mount("/api/limits",limits::routes())
        .mount("/api/orgs",orgs::routes())
        .register("/",catchers!(no_auth))
        .register("/api",catchers!(no_auth_api))
        .attach(AdHoc::config::<Config>())
        .attach(Template::fairing())
}
