#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dataregi::rocket().launch().await
}