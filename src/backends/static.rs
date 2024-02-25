use serde_json::{json, Value};

pub fn autocomplete(q: Option<String>, r#type: Option<String>, fuzzy: Option<String>, caretpos: Option<String>, per_side: Option<String>) -> Value {
    json!(
        [
            {
              "type": "vejnavn",
              "tekst": "Maribo Landevej ",
              "forslagstekst": "Maribo Landevej",
              "caretpos": 16,
              "data": {
                "navn": "Maribo Landevej",
                "href": "https://api.dataforsyningen.dk/vejnavne/Maribo%20Landevej"
              }
            },
            {
              "type": "vejnavn",
              "tekst": "Maribovej ",
              "forslagstekst": "Maribovej",
              "caretpos": 10,
              "data": {
                "navn": "Maribovej",
                "href": "https://api.dataforsyningen.dk/vejnavne/Maribovej"
              }
            },
            {
              "type": "vejnavn",
              "tekst": "Gl. Maribovej ",
              "forslagstekst": "Gl. Maribovej",
              "caretpos": 14,
              "data": {
                "navn": "Gl. Maribovej",
                "href": "https://api.dataforsyningen.dk/vejnavne/Gl.%20Maribovej"
              }
            }
          ]           
    )
}
