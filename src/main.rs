#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::serde::{Serialize, Deserialize};
use std::fs;

#[derive(FromForm, Serialize, Deserialize)]
struct UserData {
    username: String,
    email: String,
}

#[post("/submit", data = "<user_form>")]
fn submit(user_form: Form<UserData>) -> Redirect {
    let user_data = &user_form.into_inner();

    let serialized = serde_json::to_string(&user_data).unwrap();
    let _ = fs::write("user_data.json", serialized); // not async, no .await
    Redirect::to("/")
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to QuantumCoin Backend 🚀"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, submit])
        .mount("/static", FileServer::from(relative!("static")))
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),  // ✅ FIX for Render binding
            port: 8080, // Default Render port
            ..rocket::Config::default()
        })
}