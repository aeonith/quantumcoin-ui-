use rocket::form::{Form, FromForm};
use rocket::fs::TempFile;
use rocket::fs::NamedFile;
use rocket::serde::{Serialize, Deserialize};
use rocket::tokio::fs;
use std::path::PathBuf;

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

#[post("/signup", data = "<form>")]
pub async fn handle_signup(mut form: Form<SignupForm<'_>>) -> String {
    // Save ID file
    let user_dir = format!("users/{}", form.username);
    let _ = fs::create_dir_all(&user_dir).await;

    let id_path = format!("{}/id_uploaded.pdf", &user_dir);
    if let Err(e) = form.id_doc.persist_to(&id_path).await {
        return format!("Failed to save ID: {}", e);
    }

    // Save user info
    let user_data = format!(
        "Username: {}\nEmail: {}\nRevStop: {}\n2FA: {}\n",
        form.username,
        form.email,
        form.revstop.unwrap_or(false),
        form._2fa.unwrap_or(false)
    );

    let info_path = format!("{}/info.txt", &user_dir);
    if let Err(e) = fs::write(&info_path, user_data).await {
        return format!("Failed to save user info: {}", e);
    }

    // TODO: Trigger wallet creation + RevStop password + key save here

    format!("✅ Signup complete for {}", form.username)
}