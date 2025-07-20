#[macro_use] extern crate rocket;

use rocket::fs::{FileServer, relative};
use rocket_dyn_templates::Template;
use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::serde::{Serialize, Deserialize};
use rocket::tokio::fs;
use std::collections::HashMap;

// ========== STRUCTS ==========

#[derive(FromForm)]
pub struct SignupForm<'r> {
    pub username: &'r str,
    pub email: &'r str,
    pub password: &'r str,
    #[field(name = "id_document")]
    pub id_doc: TempFile<'r>,
    pub revstop: Option<bool>,
    pub _2fa: Option<bool>,
}

// ========== ROUTES ==========

#[get("/")]
fn index() -> Template {
    Template::render("index", &HashMap::<String, String>::new())
}

#[get("/signup")]
fn signup_form() -> Template {
    Template::render("signup", &HashMap::<String, String>::new())
}

#[post("/signup", data = "<form>")]
async fn handle_signup(mut form: Form<SignupForm<'_>>) -> Template {
    let user_dir = format!("users/{}", form.username);
    let _ = fs::create_dir_all(&user_dir).await;

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

// ========== LAUNCH ==========

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![
            index,
            signup_form,
            handle_signup
        ])
        .mount("/static", FileServer::from(relative!("static")))
        .attach(Template::fairing())
}