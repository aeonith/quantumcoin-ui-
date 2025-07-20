#[macro_use] extern crate rocket;

use rocket::form::Form;
use std::fs;
use std::io::Write;

#[derive(FromForm)]
struct UserData {
    name: String,
    email: String,
}

#[post("/submit", data = "<form_data>")]
fn submit(form_data: Form<UserData>) -> &'static str {
    let info_path = "user_data.txt";
    let user_data = format!("Name: {}\nEmail: {}\n\n", form_data.name, form_data.email);

    // Use synchronous fs::write (not async, so no .await!)
    if let Err(e) = fs::write(info_path, user_data) {
        eprintln!("Failed to write file: {}", e);
        return "Failed to save data.";
    }

    "User data submitted successfully!"
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to QuantumCoin Backend"
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index, submit])
}