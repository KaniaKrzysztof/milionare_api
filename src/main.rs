#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate reqwest;
extern crate serde;

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
        let _ = download_and_upload::fill_db_with_questions();
        return "upload sucess".to_string();
    } else {
        return "upload failed".to_string();
    }
}

fn main() {
    dotenv().ok();
    rocket::ignite()
        .mount("/", routes![api])
        .mount("/", routes![index])
        .mount("/", routes![upload])
        .launch();
}
