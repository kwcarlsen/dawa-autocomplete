use dawa_autocomplete::SizeOf;
use std::fmt::Display;
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, SizeOf)]
pub struct Address {
    pub id: Uuid,
    pub street_code: i32,
    pub municipal_code: i32,
    pub street: String,
    pub number: String,
    pub floor: String,
    pub door: String,
    pub zip: String,
    pub placename: String,
    pub city: String,
}

// impl Display for Address {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.to_string())
//     }
// }

impl From<Address> for String {
    fn from(address: Address) -> Self {
        address.display_name()
    }
}

impl Address {
    pub fn display_name(&self) -> String {
        let mut display_name = format!("{} {}", self.street, self.number);
        if !self.floor.is_empty() {
            display_name.push_str(&format!(", {}.", self.floor));
        }
        if self.floor.is_empty() && !self.door.is_empty() {
            display_name.push_str(&format!(","));
        }
        if !self.door.is_empty() {
            display_name.push_str(&format!(" {}", self.door));
        }
        if !self.placename.is_empty() {
            display_name.push_str(&format!(", {}", self.placename));
        }
        display_name.push_str(&format!(", {} {}", self.zip, self.city));
        display_name
    }

    pub fn access_address_name(&self) -> String {
        format!(
            "{} {}, {},{}",
            self.street, self.number, self.zip, self.city
        )
    }
}

impl Default for Address {
    fn default() -> Self {
        Address {
            id: Uuid::default(),
            street_code: 0,
            municipal_code: 0,
            street: "".to_string(),
            number: "".to_string(),
            floor: "".to_string(),
            door: "".to_string(),
            zip: "".to_string(),
            placename: "".to_string(),
            city: "".to_string(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::address::Address;

    #[test]
    fn test_display_name() {
        let address = Address {
            id: Uuid::default(),
            street_code: 1,
            municipal_code: 1,
            street: "Kronprinsesse Sofies Vej".to_string(),
            number: "1".to_string(),
            floor: "st.".to_string(),
            door: "".to_string(),
            placename: "".to_string(),
            zip: "2000".to_string(),
            city: "Frederiksberg".to_string(),
        };

        assert_eq!(
            address.display_name(),
            "Kronprinsesse Sofies Vej 1, Frederiksberg"
        );
    }
}
