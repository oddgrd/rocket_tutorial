#[macro_use]
extern crate rocket;
use rocket::tokio::time::{sleep, Duration};
mod api_key;

#[cfg(test)]
mod tests;

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[get("/<name>")]
fn greeting(name: String) -> String {
    format!("Hello {}", name)
}

#[get("/hello?<name>&<salutation>")]
fn query_greeting(name: String, salutation: Option<String>) -> String {
    match salutation {
        Some(s) => format!("{} {}", s, name),
        None => format!("Hello {}", name),
    }
}

#[get("/delay/<seconds>")]
async fn delay(seconds: u64) -> String {
    sleep(Duration::from_secs(seconds)).await;
    format!("Hello, world {} seconds later.", seconds)
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/api", routes![index, greeting, query_greeting, delay])
}
