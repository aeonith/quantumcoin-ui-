#[macro_use] extern crate rocket;

use rocket_dyn_templates::Template;
use std::collections::HashMap;

#[get("/")]
fn index() -> Template {
    let mut context = HashMap::new();
    context.insert("title", "QuantumCoin™");
    context.insert("tagline", "The Future of Quantum-Resistant Currency");
    context.insert("description", "QuantumCoin™ is a next-generation cryptocurrency offering secure, quantum-resistant transactions, real-time price tracking, and a fully integrated wallet and blockchain platform.");
    Template::render("index", &context)
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![index])
        .attach(Template::fairing())
}