#[macro_use] extern crate rocket;

use rocket::form::Form;
use rocket::fs::{FileServer, relative};
use rocket::response::Redirect;
use rocket_dyn_templates::Template;
use std::collections::HashMap;

#[derive(FromForm)]
struct SignupData {
    username: String,
    email: String,
    password: String,
}

#[get("/signup")]
fn signup_form() -> Template {
    Template::render("signup", &HashMap::<String, String>::new())
}

#[post("/signup", data = "<form_data>")]
fn process_signup(form_data: Form<SignupData>) -> Redirect {
    println!("📥 New signup: {} | {}", form_data.username, form_data.email);
    // Later: Add DB, wallet key gen, RevStop, etc.
    Redirect::to("/signup_success")
}

#[get("/signup_success")]
fn signup_success() -> &'static str {
    "✅ Signup complete! (Next: generate wallet, KYC, RevStop...)"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![signup_form, process_signup, signup_success])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}