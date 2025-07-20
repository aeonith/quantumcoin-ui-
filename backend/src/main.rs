#[macro_use] extern crate rocket;

use rocket::form::Form;
use rocket_dyn_templates::{Template, context};

#[derive(FromForm)]
struct SignupForm {
    username: String,
    email: String,
    password: String,
    id_number: String,
}

#[get("/signup")]
fn signup_page() -> Template {
    Template::render("signup", context! {})
}

#[post("/signup", data = "<form_data>")]
fn signup_submit(form_data: Form<SignupForm>) -> String {
    let data = form_data.into_inner();
    // In production, encrypt + store in database here.
    format!("User {} with email {} submitted for KYC!", data.username, data.email)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![signup_page, signup_submit])
        .attach(Template::fairing())
}