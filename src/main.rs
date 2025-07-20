#[macro_use]
extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket::form::Form;
use rocket::response::Redirect;
use rocket::serde::{Serialize, Deserialize};
use std::fs;

/// Struct to handle submitted user data
#[derive(FromForm, Serialize, Deserialize)]
struct UserData {
    username: String,
    email: String,
}

/// POST route to handle form submission and save to a file
#[post("/submit", data = "<user_form>")]
fn submit(user_form: Form<UserData>) -> Redirect {
    let user_data = &user_form.into_inner();

    let serialized = serde_json::to_string(&user_data).expect("Failed to serialize form data");
    let _ = fs::write("user_data.json", serialized).expect("Failed to write user_data.json");

    Redirect::to("/")
}

/// GET route for homepage
#[get("/")]
fn index() -> &'static str {
    "Welcome to QuantumCoin Backend 🚀"
}

/// Rocket launch configuration
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, submit])
        .mount("/static", FileServer::from(relative!("static")))
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 8080, // Render default port
            ..rocket::Config::default()
        })
}