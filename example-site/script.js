"use strict"
dawaAutocomplete.dawaAutocomplete( document.getElementById("adresse"), {
  select: function(selected) {
    document.getElementById("valgtadresse").innerHTML= selected.tekst;
  },
  baseUrl: "http://localhost:8000",
});
dawaAutocomplete.dawaAutocomplete( document.getElementById("adresse-dawa"), {
  select: function(selected) {
    document.getElementById("valgtadresse-dawa").innerHTML= selected.tekst;
  },
});
