#[macro_use]
extern crate rocket;

use rocket::fs::{FileServer, relative, TempFile};
use rocket::form::Form;
use rocket::response::{Redirect, content::RawHtml};
use rocket::serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

mod wallet;
mod revstop;

#[derive(FromForm, Serialize, Deserialize, Clone)]
struct UserData {
    username: String,
    email: String,
    password: String,
}

#[post("/register", data = "<reg_form>")]
fn register(reg_form: Form<UserData>) -> Redirect {
    let new_user = reg_form.into_inner();

    let mut users: Vec<UserData> = if let Ok(content) = fs::read_to_string("users.json") {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    users.push(new_user);
    let serialized = serde_json::to_string_pretty(&users).unwrap();
    fs::write("users.json", serialized).expect("Failed to write users.json");

    Redirect::to("/static/index.html")
}

#[derive(FromForm)]
struct LoginForm {
    username: String,
    password: String,
}

#[post("/login", data = "<login_form>")]
fn login(login_form: Form<LoginForm>) -> String {
    let creds = login_form.into_inner();

    if let Ok(content) = fs::read_to_string("users.json") {
        let users: Vec<UserData> = serde_json::from_str(&content).unwrap_or_default();
        for user in users {
            if user.username == creds.username && user.password == creds.password {
                return format!("Login successful. Welcome, {}!", user.username);
            }
        }
    }

    "Invalid credentials.".to_string()
}

#[post("/kyc", data = "<file>")]
async fn kyc_upload(mut file: TempFile<'_>) -> &'static str {
    let save_path = format!("uploads/{}", file.name().unwrap_or("kyc_file"));
    if let Err(_) = file.persist_to(save_path).await {
        return "Upload failed.";
    }
    "KYC upload successful."
}

#[get("/keys")]
fn show_keys() -> String {
    let (pub_key, priv_key) = wallet::get_keys();
    format!("Public Key:\n{}\n\nPrivate Key:\n{}", pub_key, priv_key)
}

#[post("/revstop/toggle")]
fn toggle_revstop() -> String {
    if revstop::is_revstop_active() {
        revstop::deactivate();
        "RevStop Deactivated.".to_string()
    } else {
        revstop::activate();
        "RevStop Activated.".to_string()
    }
}

#[get("/admin")]
fn admin_panel() -> RawHtml<String> {
    let user_count = if let Ok(content) = fs::read_to_string("users.json") {
        let users: Vec<UserData> = serde_json::from_str(&content).unwrap_or_default();
        users.len()
    } else {
        0
    };

    let revstop_status = if revstop::is_revstop_active() {
        "🔒 Locked"
    } else {
        "✅ Unlocked"
    };

    let mut kyc_files = Vec::new();
    if let Ok(entries) = fs::read_dir("uploads") {
        for entry in entries.flatten() {
            if let Ok(name) = entry.file_name().into_string() {
                kyc_files.push(name);
            }
        }
    }
    kyc_files.sort();
    kyc_files.reverse();
    let kyc_list = kyc_files.iter().take(5)
        .map(|f| format!("<li>{}</li>", f))
        .collect::<Vec<_>>()
        .join("");

    let html = format!(r#"
    <html>
    <head><title>Admin Dashboard</title></head>
    <body style="background-color:#0b0b0b;color:#f0f0f0;font-family:sans-serif;">
        <h1>🛠 QuantumCoin Admin Panel</h1>
        <p><strong>User Accounts:</strong> {}</p>
        <p><strong>RevStop Status:</strong> {}</p>
        <h3>📥 Latest KYC Uploads:</h3>
        <ul>{}</ul>
        <button onclick="alert('⛏ Mining Triggered (Simulated)')">Mine Transactions</button>
        <br><br><a href='/static/index.html'>← Back to App</a>
    </body>
    </html>
    "#, user_count, revstop_status, kyc_list);

    RawHtml(html)
}

#[get("/")]
fn index() -> &'static str {
    "Welcome to QuantumCoin Backend 🚀"
}

#[launch]
fn rocket() -> _ {
    if !Path::new("uploads").exists() {
        fs::create_dir("uploads").unwrap();
    }

    rocket::build()
        .mount("/", routes![
            index,
            register,
            login,
            kyc_upload,
            show_keys,
            toggle_revstop,
            admin_panel
        ])
        .mount("/static", FileServer::from(relative!("static")))
        .configure(rocket::Config {
            address: "0.0.0.0".parse().unwrap(),
            port: 8080,
            ..rocket::Config::default()
        })
}