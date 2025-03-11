use std::{
    collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap},
    ops::Bound,
    path::Path,
    sync::Arc,
    time::Instant,
};

use crate::token_index::{DawaUuid, TokenIndex};
use crate::{address::Address, size_of::SizeOf};
use log::info;
use uuid::Uuid;

const ADDRESS_FILENAME: &str = "address.csv";
const DAWA_ADDRESS_FILENAME: &str = "../addresser.csv";

pub struct Municipality {
    pub code: i32,
    pub name: String,
    pub zip: String,
}

pub struct Query {
    pub q: String,
    pub r#type: Option<String>,
    pub fuzzy: Option<String>,
    pub caretpos: Option<String>,
    pub per_side: Option<i32>,
}

pub enum SearchMode {
    Street,
    AccessAddress,
    Address,
    None,
}

#[derive(Debug, Default)]
pub struct QueryElement {
    street_name: Option<String>,
    number: Option<String>,
    floor: Option<String>,
    door: Option<String>,
    zip: Option<String>,
    city: Option<String>,
}

impl From<&String> for QueryElement {
    fn from(query: &String) -> QueryElement {
        let mut query_element = QueryElement::default();
        let parts: Vec<&str> = query.split(' ').collect();
        for part in parts.iter() {
            if part.parse::<i32>().is_ok() {
                if part.len() == 4 {
                    query_element.zip = Some(part.to_string());
                } else {
                    query_element.number = Some(part.to_string());
                }
            }
        }
        query_element.street_name = Some(parts[0].to_string());
        // query_element.number = Some(parts[1].to_string());
        query_element
    }
}

impl QueryElement {
    pub fn get_search_mode(
        &self,
        startfra: Option<String>,
        access_address_id: &Option<String>,
    ) -> SearchMode {
        if access_address_id.is_some() {
            return SearchMode::Address;
        }
        if startfra.is_some() {
            return SearchMode::AccessAddress;
        }
        if self.street_name.is_some() && self.number.is_some() {
            if self.floor.is_some() && self.door.is_some() {
                return SearchMode::Address;
            }
            return SearchMode::AccessAddress;
        }
        return SearchMode::Street;
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
struct AccessAddressIndicator {
    pub municipal_code: i32,
    pub street_code: i32,
}

impl AccessAddressIndicator {
    pub fn new(municipal_code: i32, street_code: i32) -> AccessAddressIndicator {
        AccessAddressIndicator {
            municipal_code,
            street_code,
        }
    }
}

pub struct AccessAddress {
    pub name: String,
    pub municipal_code: i32,
    pub address: BTreeMap<String, Vec<VerticalAddress>>,
}

pub struct VerticalAddress {
    pub floor: String,
    pub door: String,
}

impl From<&AccessAddress> for Address {
    fn from(access_address: &AccessAddress) -> Address {
        Address {
            id: Uuid::default(),
            street_code: 1,
            municipal_code: access_address.municipal_code,
            street: access_address.name.clone(),
            number: "1".to_string(),
            floor: "1".to_string(),
            door: "1".to_string(),
            placename: "1".to_string(),
            city: "1".to_string(),
            zip: "1".to_string(),
        }
    }
}

pub struct AddressCompleter {
    access_address: BTreeMap<AccessAddressIndicator, AccessAddress>,
    municipalities: BTreeMap<i32, Municipality>,
    index: BTreeMap<String, AccessAddressIndicator>,
    token_index: TokenIndex,
    trigrams: BTreeMap<String, Vec<AccessAddressIndicator>>,

    // addresses: BTreeMap<DawaUuid, Arc<Address>>,
    addresses: BTreeMap<Uuid, Vec<Arc<Address>>>,
    street_names: BTreeMap<String, Arc<Address>>,
    access_addresses: BTreeMap<String, Arc<Address>>,
}

impl AddressCompleter {
    pub fn new() -> AddressCompleter {
        AddressCompleter {
            access_address: BTreeMap::new(),
            municipalities: BTreeMap::new(),
            index: BTreeMap::new(),
            trigrams: BTreeMap::new(),
            token_index: TokenIndex::new(),
            addresses: BTreeMap::new(),
            street_names: BTreeMap::new(),
            access_addresses: BTreeMap::new(),
        }
    }

    pub fn add_address(&mut self, address: Address) {
        let access_address_indicator =
            AccessAddressIndicator::new(address.municipal_code, address.street_code);
        self.access_address
            .entry(access_address_indicator)
            .or_insert(AccessAddress {
                name: address.street.clone(),
                municipal_code: address.municipal_code,
                address: BTreeMap::new(),
            })
            .address
            .entry(address.number.parse().unwrap())
            .or_default()
            .push(VerticalAddress {
                floor: address.floor.parse().unwrap(),
                door: address.door.parse().unwrap(),
            });

        self.municipalities
            .entry(address.municipal_code)
            .or_insert(Municipality {
                code: address.municipal_code,
                name: address.city.clone(),
                zip: address.zip.clone(),
            });
    }

    pub fn init() -> AddressCompleter {
        let mut address_completer = AddressCompleter::new();

        if !Path::new(ADDRESS_FILENAME).exists() && Path::new(DAWA_ADDRESS_FILENAME).exists() {
            info!("Converting addresses");
            address_completer.convert_from_dawa_export(DAWA_ADDRESS_FILENAME, ADDRESS_FILENAME);
        }

        if Path::new(ADDRESS_FILENAME).exists() {
            info!("Loading addresses");
            address_completer.load(ADDRESS_FILENAME);
            return address_completer;
        }
        panic!("File not found, use curl https://api.dataforsyningen.dk/adresser?format=csv > ../addresser.csv");
    }

    pub fn convert_from_dawa_export(&mut self, path: &str, dest: &str) {
        let mut rdr = csv::Reader::from_path(path).unwrap();
        let mut wrt = csv::Writer::from_path(dest).unwrap();
        for (count, result) in rdr.records().into_iter().enumerate() {
            let record = result.unwrap();
            wrt.write_record([
                record.get(0).unwrap().to_string(),
                record.get(4).unwrap().to_string(),
                record.get(15).unwrap().to_string(),
                record.get(5).unwrap().to_string(),
                record.get(7).unwrap().to_string(),
                record.get(8).unwrap().to_string(),
                record.get(9).unwrap().to_string(),
                record.get(10).unwrap().to_string(),
                record.get(12).unwrap().to_string(),
                record.get(11).unwrap().to_string(),
            ])
            .unwrap();
            if (count % 300000) == 0 {
                info!("Converted {} addresses", count);
            }
        }
    }

    pub fn save(&mut self, path: &str) {
        let mut wrt = csv::Writer::from_path(path).unwrap();
        let mut count = 0;
        for (address_indicator, address) in self.access_address.iter() {
            let municipality = self
                .municipalities
                .get(&address_indicator.municipal_code)
                .unwrap();
            for (number, vertical_address) in &address.address {
                for va in vertical_address {
                    wrt.write_record([
                        "1",
                        &address_indicator.street_code.to_string(),
                        &address_indicator.municipal_code.to_string(),
                        &address.name,
                        number,
                        &va.floor,
                        &va.door,
                        &municipality.zip,
                        &municipality.name,
                        "none",
                    ])
                    .unwrap();
                }
                count += 1;
                if (count % 300000) == 0 {
                    println!("Wrote {} addresses", count);
                }
            }
        }
    }

    pub fn load(&mut self, path: &str) {
        let mut rdr = csv::Reader::from_path(path).unwrap();
        for (count, result) in rdr.records().enumerate() {
            let record = result.unwrap();
            let street: String = record.get(3).unwrap().parse().unwrap();
            let address = Address {
                id: record.get(0).unwrap().parse().unwrap(),
                street_code: record.get(1).unwrap().parse().unwrap(),
                municipal_code: record.get(2).unwrap().parse().unwrap(),
                street: street.clone(),
                number: record.get(4).unwrap().parse().unwrap(),
                floor: record.get(5).unwrap().parse().unwrap(),
                door: record.get(6).unwrap().parse().unwrap(),
                placename: record.get(7).unwrap().parse().unwrap(),
                city: record.get(8).unwrap().parse().unwrap(),
                zip: record.get(9).unwrap().parse().unwrap(),
            };
            // println!("{}", address.display_name());
            self.add_address(address.clone());
            let dawa_uuid = DawaUuid::new(record.get(0).unwrap().parse().unwrap());
            let aaddress = Arc::new(address);
            // self.addresses.insert(dawa_uuid.clone(), aaddress.clone());
            self.street_names
                .insert(street.to_lowercase(), aaddress.clone());
            self.access_addresses.insert(
                aaddress.access_address_name().to_lowercase(),
                aaddress.clone(),
            );
            self.addresses
                .entry(aaddress.id)
                .or_insert(Vec::new())
                .push(aaddress.clone());
            // let a = self.addresses.get(&dawa_uuid).unwrap();
            // self.token_index.insert(a.display_name(), a);
            // self.addresses.insert(count, address);
            if (count % 300000) == 0 {
                info!("Read {} addresses", count);
            }
        }
        self.build_indexes();

        info!(
            "Size of self.addresses:   {}",
            self.addresses.size_of() / 1024 / 1024
        );
        info!(
            "Size of self.token_index: {}",
            self.token_index.size_of() / 1024 / 1024
        );

        // let search = "15 maribovej 2500";
        // let start = Instant::now();
        // let result = self.token_index.search(search).unwrap();
        // let elapsed = start.elapsed();
        // info!("Search for {} took {:?}", search, elapsed);
        // for s in result {
        //     info!("{:?}", s);
        // }

        // let search = "maribovej";
        // let start = Instant::now();
        // let result = self.find_street(search.to_string(), 10);
        // let elapsed = start.elapsed();
        // info!("Search for {} took {:?}", search, elapsed);
        // for s in result {
        //     info!("{:?}", s);
        // }
    }

    pub fn find_access_address(&self, display_name: String, count: i32) -> Vec<Arc<Address>> {
        let mut result = Vec::new();

        let mut cursor = self
            .access_addresses
            .lower_bound(Bound::Included(&display_name.to_lowercase()));
        for _ in 0..count {
            if let Some((_, address)) = cursor.next() {
                result.push(address.clone());
            }
        }
        result
    }

    pub fn find_address(
        &self,
        display_name: &String,
        access_address_id: &Option<String>,
        count: i32,
    ) -> Vec<Arc<Address>> {
        let mut result = Vec::new();

        let uuid: Uuid = Uuid::parse_str(access_address_id.as_ref().unwrap().as_str()).unwrap();
        let cursor = self.addresses.get(&uuid);
        if let Some(cursor) = cursor {
            for a in cursor {
                result.push(a.clone());
            }
        }
        result
    }

    pub fn find_street(&self, display_name: String, count: i32) -> Vec<String> {
        let mut result = Vec::new();

        let mut cursor = self
            .street_names
            .lower_bound(Bound::Included(&display_name.to_lowercase()));
        for _ in 0..count {
            if let Some((_, address)) = cursor.next() {
                result.push(address.street.clone() + " ");
            }
        }

        // let mut trigram_matches = HashMap::new();

        // for w in display_name.into_bytes().windows(3) {
        //     let trigram = String::from_utf8_lossy(w).to_lowercase();
        //     if let Some(ids) = self.trigrams.get(&trigram) {
        //         for id in ids {
        //             let count = trigram_matches.entry(id).or_insert(0);
        //             *count += 1;
        //         }
        //     }
        // }
        // let mut heap = BinaryHeap::new();
        // for (id, count) in trigram_matches {
        //     heap.push((count, id));
        // }

        // for _ in 0..count {
        //     if let Some((count, id)) = heap.pop() {
        //         if let Some(a) = self.access_address.get(id) {
        //             println!("Count: {}, id: {:?}, address: {}", count, id, a.name);
        //             result.push(a.into());
        //         }
        //     }
        // }

        result
    }

    fn build_indexes(&mut self) {
        let start = Instant::now();
        debug!("Building indexes");
        for (count, (aai, access_address)) in self.access_address.iter().enumerate() {
            self.index
                .insert(access_address.name.clone().to_lowercase(), *aai);

            for w in access_address
                .name
                .clone()
                .to_lowercase()
                .into_bytes()
                .windows(3)
            {
                let trigram = String::from_utf8_lossy(w);
                self.trigrams
                    .entry(trigram.to_string())
                    .or_default()
                    .push(*aai);
            }

            if (count % 300000) == 0 {
                println!("Indexed {} addresses", count);
            }
        }
        let elapsed = start.elapsed();
        debug!("Indexes built in {:?}", elapsed);
        // println!("{:?}", self.trigrams);
    }
}

/**
 * Strategy:
 *   Stemming?
 *   replace all special chars
 *
 */

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;

    static ADDRESS_COMPLETER: Lazy<AddressCompleter> = Lazy::new(AddressCompleter::init);

    #[test]
    fn test_find_address() {}

    #[test]
    fn test_address_lookup() {
        let mut address_completer = AddressCompleter::new();
    }
}
