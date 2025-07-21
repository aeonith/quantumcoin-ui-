use actix_web::{web, HttpResponse, Responder, get, post, HttpRequest};
use std::collections::HashMap;
use std::sync::Mutex;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use lazy_static::lazy_static;
use std::fs::OpenOptions;
use std::io::Write;

lazy_static! {
    static ref VERIFICATION_CODES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

fn generate_code() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(6).map(char::from).collect()
}

fn log_kyc_submission(email: &str) {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open("kyc_submissions.log")
        .unwrap();
    writeln!(file, "KYC verified: {}", email).unwrap();
}

#[get("/kyc")]
pub async fn kyc_form() -> impl Responder {
    let html = r#"
        <html>
            <head><title>KYC Verification</title></head>
            <body>
                <h2>KYC Verification</h2>
                <form action="/kyc/send-code" method="post">
                    Email: <input type="email" name="email" required>
                    <button type="submit">Send Code</button>
                </form>
                <br>
                <form action="/kyc/verify" method="post">
                    Email: <input type="email" name="email" required><br>
                    Code: <input type="text" name="code" required><br>
                    <button type="submit">Verify Code</button>
                </form>
            </body>
        </html>
    "#;
    HttpResponse::Ok().content_type("text/html").body(html)
}

#[post("/kyc/send-code")]
pub async fn send_code(form: web::Form<HashMap<String, String>>) -> impl Responder {
    let email = form.get("email").unwrap_or(&"".to_string()).clone();
    let code = generate_code();
    VERIFICATION_CODES.lock().unwrap().insert(email.clone(), code.clone());

    // Replace this with actual email logic
    println!("Sending code '{}' to email '{}'", code, email);

    HttpResponse::Ok().body(format!(
        "Verification code sent to {}. Check your inbox!",
        email
    ))
}

#[post("/kyc/verify")]
pub async fn verify_code(form: web::Form<HashMap<String, String>>) -> impl Responder {
    let email = form.get("email").unwrap_or(&"".to_string()).clone();
    let input_code = form.get("code").unwrap_or(&"".to_string()).clone();

    let stored_code = VERIFICATION_CODES.lock().unwrap().get(&email).cloned();

    match stored_code {
        Some(code) if code == input_code => {
            log_kyc_submission(&email);
            HttpResponse::Ok().body("✅ KYC Verification successful!")
        },
        _ => HttpResponse::Unauthorized().body("❌ Invalid verification code."),
    }
}