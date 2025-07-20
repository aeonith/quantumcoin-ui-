#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket::form::Form;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use std::fs;
use rocket::tokio::fs::File;
use rocket::tokio::io::AsyncWriteExt;

#[derive(FromForm)]
struct SignupForm {
    username: String,
    email: String,
    revstop: Option<bool>,
    _2fa: Option<bool>,
    id_doc: rocket::fs::TempFile<'static>,
}

#[get("/")]
fn index() -> Template {
    let context = HashMap::<String, String>::new();
    Template::render("signup", &context)
}

#[get("/signup")]
fn signup_form() -> Template {
    let context = HashMap::<String, String>::new();
    Template::render("signup", &context)
}

#[post("/signup", data = "<form>")]
async fn handle_signup(mut form: Form<SignupForm>) -> Template {
    let user_dir = format!("user_data/{}", form.username);
    if let Err(e) = fs::create_dir_all(&user_dir) {
        println!("Failed to create user dir: {}", e);
    }

    let id_path = format!("{}/id_uploaded.pdf", &user_dir);
    if let Err(e) = form.id_doc.persist_to(&id_path).await {
        println!("Failed to save ID: {}", e);
    }

    let user_data = format!(
        "Username: {}\nEmail: {}\nRevStop: {}\n2FA: {}\n",
        form.username,
        form.email,
        form.revstop.unwrap_or(false),
        form._2fa.unwrap_or(false)
    );

    let info_path = format!("{}/info.txt", &user_dir);
    if let Err(e) = fs::write(&info_path, user_data).await {
        println!("Failed to save user info: {}", e);
    }

    // TODO: Trigger wallet + RevStop + admin hooks here
    let mut context = HashMap::new();
    context.insert("username", form.username.to_string());
    Template::render("success", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            index,
            signup_form,
            handle_signup
        ])
        .attach(Template::fairing())
}