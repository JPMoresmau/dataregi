
#[macro_use] extern crate rocket;

use rocket::response::status::NotFound;
use rocket::fs::{NamedFile,relative};
use std::path::{Path,PathBuf};


#[get("/<path..>")]
async fn static_files(path: PathBuf) -> Result<NamedFile, NotFound<String>> {
    let mut path = Path::new(relative!("site")).join(path);
    if path.is_dir() {
        path.push("static/index.html");
    }

    NamedFile::open(path).await.map_err(|e| NotFound(e.to_string()))
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![static_files])
}

