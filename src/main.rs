#![feature(btree_cursors)]

#[macro_use]
extern crate rocket;

use address_completer::{QueryElement, SearchMode};
// use dawa_autocomplete::size_of::SizeOf;
use rocket::State;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::{Header, Status},
    serde::json::Value,
    Request, Response,
};
use serde_json::json;

mod address;
mod address_completer;
mod backends;
pub mod size_of;
mod token_index;

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
#[get("/autocomplete?<q>&<type>&<fuzzy>&<caretpos>&<per_side>&<startfra>&<adgangsadresseid>")]
fn autocomplete(
    completer: &State<address_completer::AddressCompleter>,
    q: String,
    r#type: Option<String>,
    fuzzy: Option<String>,
    caretpos: Option<String>,
    per_side: Option<i32>,
    startfra: Option<String>,
    adgangsadresseid: Option<String>,
) -> (Status, Value) {
    let mut result = Vec::new();
    let query_element = QueryElement::from(&q);

    match query_element.get_search_mode(startfra, &adgangsadresseid) {
        SearchMode::Street => {
            for street in completer.find_street(q, per_side.unwrap_or(50)) {
                result.push(json!(
                    {
                      "type": "vejnavn",
                      "tekst": street,
                      "forslagstekst": street,
                      "caretpos": street.len(),
                      "data": {
                        "navn": street,
                        "href": format!("https://api.dataforsyningen.dk/vejnavne/{}", street)
                      }
                    }
                ));
            }
        }
        SearchMode::AccessAddress => {
            for address in completer.find_access_address(q, per_side.unwrap_or(50)) {
                result.push(json!(
                    {
                        "data": {
                          "id": "58910400-b8f1-44bf-8293-7420ee1595a8",
                          "status": 1,
                          "darstatus": 3,
                          "vejkode": "1013",
                          "vejnavn": "Maribovej",
                          "adresseringsvejnavn": "Maribovej",
                          "husnr": "1",
                          "supplerendebynavn": null,
                          "postnr": "4960",
                          "postnrnavn": "Holeby",
                          "stormodtagerpostnr": null,
                          "stormodtagerpostnrnavn": null,
                          "kommunekode": "0360",
                          "x": 11.45702023,
                          "y": 54.71182365,
                          "href": "https://api.dataforsyningen.dk/adgangsadresser/58910400-b8f1-44bf-8293-7420ee1595a8"
                        },
                        "stormodtagerpostnr": false,
                        "type": "adgangsadresse",
                        "tekst": address.access_address_name(),
                        "forslagstekst": address.access_address_name(),
                        "caretpos": address.access_address_name().len()
                      }
                ));
            }
        }
        SearchMode::Address => {
            for address in completer.find_address(&q, &adgangsadresseid, per_side.unwrap_or(50)) {
                result.push(json!(
                    {
                      "data": {
                        "id": "0a3f509f-96d7-32b8-e044-0003ba298018",
                        "status": 1,
                        "darstatus": 3,
                        "vejkode": "4640",
                        "vejnavn": "Maribovej",
                        "adresseringsvejnavn": "Maribovej",
                        "husnr": "15",
                        "etage": "st",
                        "dør": "tv",
                        "supplerendebynavn": null,
                        "postnr": "2500",
                        "postnrnavn": "Valby",
                        "stormodtagerpostnr": null,
                        "stormodtagerpostnrnavn": null,
                        "kommunekode": "0101",
                        "adgangsadresseid": "0a3f507a-c086-32b8-e044-0003ba298018",
                        "x": 12.48971377,
                        "y": 55.667307,
                        "href": "https://api.dataforsyningen.dk/adresser/0a3f509f-96d7-32b8-e044-0003ba298018"
                      },
                      "stormodtagerpostnr": false,
                      "type": "adresse",
                      "tekst": address.display_name(),
                      "forslagstekst": address.display_name(),
                      "caretpos": address.display_name().len()
                    }
                ));
            }
        }
        SearchMode::None => {}
    }

    (Status::Ok, json!(result))
}

#[launch]
fn rocket() -> _ {
    env_logger::init();

    let address_completer = address_completer::AddressCompleter::init();

    rocket::build()
        .attach(Cors)
        .manage(address_completer)
        .mount("/", routes![autocomplete])
}
