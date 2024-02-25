#![feature(btree_cursors)]

#[macro_use]
extern crate rocket;

use rocket::State;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Header, Status},
    serde::json::Value,
    Request, Response,
};
use serde_json::json;

mod address_completer;
mod backends;

struct Cors;

#[rocket::async_trait]
impl Fairing for Cors {
    fn info(&self) -> Info {
        Info {
            name: "Add CORS headers to responses",
            kind: Kind::Response,
        }
    }

    async fn on_response<'r>(&self, _request: &'r Request<'_>, response: &mut Response<'r>) {
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        response.set_header(Header::new(
            "Access-Control-Allow-Methods",
            "POST, GET, PATCH, OPTIONS",
        ));
        response.set_header(Header::new("Access-Control-Allow-Headers", "*"));
        response.set_header(Header::new("Access-Control-Allow-Credentials", "true"));
    }
}
//GET https://dawa.aws.dk/autocomplete?q=kronprinsesse&type=adresse&caretpos=0&supplerendebynavn=true&stormodtagerpostnumre=true&multilinje=true&fuzzy=    => Vejnavn
//GET https://dawa.aws.dk/autocomplete?q=maribovej 1&type=adresse&caretpos=11&supplerendebynavn=true&stormodtagerpostnumre=true&multilinje=true&fuzzy= => Adgangsadresse
//GET https://dawa.aws.dk/autocomplete?q=Kronprinsesse Sofies Vej 1, st., 2000 Frederiksberg&type=adresse&caretpos=37&supplerendebynavn=true&stormodtagerpostnumre=true&multilinje=true&fuzzy=   => Adresse
#[get("/autocomplete?<q>&<type>&<fuzzy>&<caretpos>&<per_side>")]
fn autocomplete(
    completer: &State<address_completer::AddressCompleter>,
    q: String,
    r#type: Option<String>,
    fuzzy: Option<String>,
    caretpos: Option<String>,
    per_side: Option<i32>,
) -> (Status, Value) {
    let mut result = Vec::new();

    for address in completer.find_street(q, per_side.unwrap_or(50)) {
        result.push(
        json!(
                {
                  "type": "vejnavn",
                  "tekst": address.display_name.clone(),
                  "forslagstekst": address.display_name.clone(),
                  "caretpos": 16,
                  "data": {
                    "navn": address.display_name.clone(),
                    "href": "https://api.dataforsyningen.dk/vejnavne/Maribo%20Landevej"
                  }
                }
            )
        );
    }
    (Status::Ok, json!(result))
}

#[launch]
fn rocket() -> _ {
    let address_completer = address_completer::AddressCompleter::init();

    rocket::build()
        .attach(Cors)
        .manage(address_completer)
        .mount("/", routes![autocomplete])
}
