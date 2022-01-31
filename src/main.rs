use rocket::{
    catch, catchers,
    fairing::{self, Fairing, Info, Kind},
    get,
    http::{Cookie, CookieJar, Method},
    launch, post,
    response::{content::Html, status::Created},
    routes,
    serde::json::Json,
    tokio::time::{sleep, Duration},
    uri, Build, Data, Request, Rocket, State,
};
use serde::{Deserialize, Serialize};
use std::str;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
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

#[get("/protected")]
fn protected(key: api_key::ApiKey) -> String {
    format!(
        "You are allowed to access this API because you presented key '{}'",
        key.0
    )
}

#[get("/login")]
fn login(jar: &CookieJar) {
    jar.add(Cookie::new(
        "Session",
        base64::encode("this_is_a_session_key"),
    ));
}

#[get("/session")]
fn session(jar: &CookieJar) -> &'static str {
    match jar.get("Session") {
        Some(_) => "You got the cookie!",
        None => "Sorry, no cookie!",
    }
}

type ID = usize;

#[derive(Serialize, Debug, Clone)]
struct Hero {
    id: ID,
    name: String,
    #[serde(rename(serialize = "canFly"))]
    can_fly: bool,
}

#[derive(Deserialize, Debug)]
struct NewHero {
    name: String,
    #[serde(rename(deserialize = "canFly"))]
    can_fly: bool,
}

struct HeroCount(AtomicUsize);
type HeroesMap = RwLock<HashMap<ID, Hero>>;

#[post("/heroes", format = "json", data = "<hero>")]
fn add_hero(
    hero: Json<NewHero>,
    heroes_state: &State<HeroesMap>,
    hero_count: &State<HeroCount>,
) -> Created<Json<Hero>> {
    // Generate unique hero ID
    let hid = hero_count.0.fetch_add(1, Ordering::Relaxed);

    // Build new hero
    let new_hero = Hero {
        id: hid,
        name: hero.0.name,
        can_fly: hero.0.can_fly,
    };

    // Insert new hero in hashmap
    let mut heroes = heroes_state.write().unwrap();
    heroes.insert(hid, new_hero.clone());

    // Use uri macro to generate location header
    //    (see https://rocket.rs/v0.4/guide/responses/#typed-uris)
    let location = uri!("/api", get_hero(hid));
    Created::new(location.to_string()).body(Json(new_hero))
}

// Note that we return `Option`. `None` would result in 404 (not found).
#[get("/heroes/<id>")]
fn get_hero(id: ID, heroes_state: &State<HeroesMap>) -> Option<Json<Hero>> {
    let heroes = heroes_state.read().unwrap();
    heroes.get(&id).map(|h| Json(h.clone()))
}

#[get("/heroes")]
fn get_all(heroes_state: &State<HeroesMap>) -> Json<Vec<Hero>> {
    let heroes = heroes_state.read().unwrap();
    Json(heroes.values().cloned().collect())
}

// region Catcher for 404
// Catcher for 404 errors
//    (see https://rocket.rs/v0.4/guide/requests/#error-catchers)
#[catch(404)]
fn not_found() -> Html<&'static str> {
    Html(
        r#"
        <h1>Not found</h1>
        <p>What are you looking for?</p>
    "#,
    )
}

// region Count Fairing
// Implement a fairing that counts all requests
//    (more about fairings at https://rocket.rs/v0.4/guide/fairings/#fairings)
#[derive(Default, Clone)]
struct Counter {
    get: Arc<AtomicUsize>,
    post: Arc<AtomicUsize>,
}

#[rocket::async_trait]
impl Fairing for Counter {
    fn info(&self) -> Info {
        Info {
            name: "GET/POST Counter",
            kind: Kind::Ignite | Kind::Request,
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        #[get("/api/counts")]
        fn counts(counts: &State<Counter>) -> String {
            let get_count = counts.get.load(Ordering::Relaxed);
            let post_count = counts.post.load(Ordering::Relaxed);
            format!("Get: {}\nPost: {}", get_count, post_count)
        }

        Ok(rocket.manage(self.clone()).mount("/", routes![counts]))
    }

    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        if request.method() == Method::Get {
            self.get.fetch_add(1, Ordering::Relaxed);
        } else if request.method() == Method::Post {
            self.post.fetch_add(1, Ordering::Relaxed);
        }
    }
}
// endregion
#[launch]
fn rocket() -> _ {
    rocket::build()
        .mount(
            "/api",
            routes![
                index,
                greeting,
                query_greeting,
                delay,
                protected,
                login,
                session,
                add_hero,
                get_hero,
                get_all,
            ],
        )
        .manage(RwLock::new(HashMap::<ID, Hero>::new()))
        .manage(HeroCount(AtomicUsize::new(1)))
        // Register catchers for errors.
        //    (see https://rocket.rs/v0.4/guide/requests/#error-catchers)
        .register("/", catchers![not_found])
        .attach(Counter::default())
}
