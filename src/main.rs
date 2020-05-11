#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
// extern crate dotenv;
// extern crate postgres;
extern crate reqwest;
extern crate serde;
// extern crate rocket_contrib;

use dotenv::dotenv;
use milionares_api::db;
use milionares_api::download_and_upload;
use milionares_api::question;
use rocket::http::RawStr;
use rocket_contrib::json::Json;
use std::env;

#[get("/api")]
fn api() -> Json<Vec<question::Question>> {
    let mut db_client = db::get_db_client().unwrap();

    let q = question::get_questions(&mut db_client).unwrap();

    Json(q)
}

#[get("/")]
fn index() -> String {
    "Hello millionare!".to_string()
}

#[get("/upload?<password>")]
fn upload(password: &RawStr) -> String {
    let upload_pass = env::var("UPLOAD_PASS").expect("upload password env error");
    if upload_pass == password.to_string() {
        return format!("dziala {}", password.to_string());
    } else {
        return format!("nie dziala {}", password.to_string());
    }
}

fn main() {
    dotenv().ok();
    // download_and_upload::download_questions();

    // let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    rocket::ignite()
        .mount("/", routes![api])
        .mount("/", routes![index])
        .mount("/", routes![upload])
        .launch();
}
