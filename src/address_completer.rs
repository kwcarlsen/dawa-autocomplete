use std::{
    collections::{BTreeMap, BinaryHeap, HashMap},
    ops::Bound,
    path::Path,
};

const ADDRESS_FILENAME: &str = "address-500k.csv";
const DAWA_ADDRESS_FILENAME: &str = "../address.csv";

pub struct Municipality {
    pub code: String,
    pub name: String,
    pub zip: String,
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

#[derive(Debug)]
pub struct Address {
    pub id: String,
    pub street_code: String,
    pub municipal_code: String,
    pub street: String,
    pub number: String,
    pub floor: String,
    pub door: String,
    pub zip: String,
    pub city: String,
    pub display_name: String,
}

impl From<&AccessAddress> for Address {
    fn from(access_address: &AccessAddress) -> Address {
        Address {
            id: "1".to_string(),
            street_code: "1".to_string(),
            municipal_code: access_address.municipal_code.to_string(),
            street: access_address.name.clone(),
            number: "1".to_string(),
            floor: "1".to_string(),
            door: "1".to_string(),
            city: "1".to_string(),
            zip: "1".to_string(),
            display_name: access_address.name.clone(),
        }
    }
}

pub struct AddressCompleter {
    access_address: BTreeMap<AccessAddressIndicator, AccessAddress>,
    municipalities: BTreeMap<i32, Municipality>,
    index: BTreeMap<String, AccessAddressIndicator>,
    trigrams: BTreeMap<String, Vec<AccessAddressIndicator>>,
}

impl AddressCompleter {
    pub fn new() -> AddressCompleter {
        AddressCompleter {
            access_address: BTreeMap::new(),
            municipalities: BTreeMap::new(),
            index: BTreeMap::new(),
            trigrams: BTreeMap::new(),
        }
    }

    pub fn add_address(&mut self, address: Address) {
        let access_address_indicator = AccessAddressIndicator::new(
            address.municipal_code.parse().unwrap(),
            address.street_code.parse().unwrap(),
        );
        self.access_address
            .entry(access_address_indicator)
            .or_insert(AccessAddress {
                name: address.street.clone(),
                municipal_code: address.municipal_code.parse().unwrap(),
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
            .entry(address.municipal_code.parse().unwrap())
            .or_insert(Municipality {
                code: address.municipal_code.clone(),
                name: address.city.clone(),
                zip: address.zip.clone(),
            });
    }

    pub fn init() -> AddressCompleter {
        let mut address_completer = AddressCompleter::new();

        if Path::new(ADDRESS_FILENAME).exists() {
            println!("Loading addresses");
            address_completer.load(ADDRESS_FILENAME);
            return address_completer;
        }
        if Path::new(DAWA_ADDRESS_FILENAME).exists() {
            println!("Importing addresses");
            address_completer.import_from_dawa_export(DAWA_ADDRESS_FILENAME);
            println!("Saving addresses");
            address_completer.save(ADDRESS_FILENAME);
            return address_completer;
        }
        panic!("File not found, use curl https://api.dataforsyningen.dk/adresser?format=csv > ../addresser.csv");
    }

    pub fn import_from_dawa_export(&mut self, path: &str) {
        let mut rdr = csv::Reader::from_path(path).unwrap();
        for (count, result) in rdr.records().into_iter().enumerate() {
            let record = result.unwrap();
            let address = Address {
                id: record.get(0).unwrap().to_string(),
                street_code: record.get(4).unwrap().to_string(),
                municipal_code: record.get(15).unwrap().to_string(),
                street: record.get(5).unwrap().to_string(),
                number: record.get(7).unwrap().to_string(),
                floor: record.get(8).unwrap().to_string(),
                door: record.get(9).unwrap().to_string(),
                city: record.get(12).unwrap().to_string(),
                zip: record.get(11).unwrap().to_string(),
                display_name: record.get(82).unwrap().to_string(),
            };
            self.add_address(address);
            // self.addresses.insert(count, address);
            if (count % 300000) == 0 {
                println!("Imported {} addresses", count);
            }
        }
        self.build_indexes();
    }

    pub fn save(&mut self, path: &str) {
        let mut wrt = csv::Writer::from_path(path).unwrap();
        let mut count = 0;
        for (address_indicator, address) in self.access_address.iter() {
            let unicipality = self
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
                        &unicipality.zip,
                        &unicipality.name,
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
            let address = Address {
                id: record.get(0).unwrap().parse().unwrap(),
                street_code: record.get(1).unwrap().parse().unwrap(),
                municipal_code: record.get(2).unwrap().parse().unwrap(),
                street: record.get(3).unwrap().parse().unwrap(),
                number: record.get(4).unwrap().parse().unwrap(),
                floor: record.get(5).unwrap().parse().unwrap(),
                door: record.get(6).unwrap().parse().unwrap(),
                city: record.get(7).unwrap().parse().unwrap(),
                zip: record.get(8).unwrap().parse().unwrap(),
                display_name: record.get(9).unwrap().parse().unwrap(),
            };
            self.add_address(address);
            // self.addresses.insert(count, address);
            if (count % 300000) == 0 {
                println!("Read {} addresses", count);
            }
        }
        self.build_indexes();
    }

    pub fn _find_street() {}

    pub fn _find_access_address() {}

    pub fn _find_address() {}

    pub fn find_street(&self, display_name: String, count: i32) -> Vec<Address> {
        let mut result = Vec::new();

        let mut cursor = self
            .index
            .lower_bound(Bound::Included(&display_name.to_lowercase()));
        for _ in 0..3 {
            if let Some((_, v)) = cursor.next() {
                if let Some(a) = self.access_address.get(v) {
                    result.push(a.into());
                }
            }
        }

        let mut trigram_matches = HashMap::new();

        for w in display_name.into_bytes().windows(3) {
            let trigram = String::from_utf8_lossy(w).to_lowercase();
            if let Some(ids) = self.trigrams.get(&trigram) {
                for id in ids {
                    let count = trigram_matches.entry(id).or_insert(0);
                    *count += 1;
                }
            }
        }
        let mut heap = BinaryHeap::new();
        for (id, count) in trigram_matches {
            heap.push((count, id));
        }

        for _ in 0..count {
            if let Some((count, id)) = heap.pop() {
                if let Some(a) = self.access_address.get(id) {
                    println!("Count: {}, id: {:?}, address: {}", count, id, a.name);
                    result.push(a.into());
                }
            }
        }

        result
    }

    fn build_indexes(&mut self) {
        println!("Building indexes");
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
    fn test_find_address() {
    }

    #[test]
    fn test_address_lookup() {
        let mut address_completer = AddressCompleter::new();
    }
}
