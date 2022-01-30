use rocket::http::Status;
use rocket::local::blocking::Client;

#[test]
fn hello_world() {
    let client = Client::tracked(super::rocket()).expect("valid rocket instance");
    let response = client.get("/api").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Hello, world!".into()));
}

#[test]
fn greeting() {
    let client = Client::tracked(super::rocket()).expect("valid rocket instance");
    let response = client.get("/api/Odd").dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Hello Odd".into()));
}
#[test]
fn query_greeting() {
    let client = Client::tracked(super::rocket()).expect("valid rocket instance");
    let response = client
        .get("/api/hello?name=Oddbj%C3%B8rn&salutation=Greetings")
        .dispatch();

    assert_eq!(response.status(), Status::Ok);
    assert_eq!(response.into_string(), Some("Greetings Oddbjørn".into()));
}
