use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use dawa_autocomplete::SizeOf;

use crate::address::Address;

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Clone, SizeOf)]
pub struct DawaUuid {
    uuid: String,
}

impl From<String> for DawaUuid {
    fn from(uuid: String) -> Self {
        DawaUuid { uuid }
    }
}

impl From<&str> for DawaUuid {
    fn from(uuid: &str) -> Self {
        DawaUuid {
            uuid: uuid.to_string(),
        }
    }
}

impl Default for DawaUuid {
    fn default() -> Self {
        DawaUuid {
            uuid: "default".to_string(),
        }
    }
}

impl DawaUuid {
    pub fn new(uuid: String) -> DawaUuid {
        DawaUuid { uuid }
    }
}

#[derive(SizeOf)]
pub struct TokenIndex {
    pub token_index: BTreeMap<String, BTreeSet<Arc<Address>>>,
}

impl TokenIndex {
    pub fn new() -> Self {
        TokenIndex {
            token_index: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, string: String, address: &Arc<Address>) {
        debug!("{}", string);
        let lowercase = string.to_lowercase().replace(",", "").replace(".", "");
        let tokens = lowercase.split_whitespace();
        for token in tokens {
            self.token_index
                .entry(token.to_string())
                .or_insert(BTreeSet::new())
                .insert(address.clone());
        }
    }

    pub fn search(&self, query: &str) -> Option<BTreeSet<Arc<Address>>> {
        let mut result: Option<BTreeSet<Arc<Address>>> = None;
        let mut tokens: Vec<&str> = query.split_whitespace().collect();
        tokens.sort_by_key(|token| -1 * token.len() as i64);
        for token in tokens {
            if let Some(uuids) = self.token_index.get(token) {
                if let Some(r) = result {
                    result = Some(r.intersection(uuids).cloned().collect());
                } else {
                    result = Some(uuids.clone());
                }
            } else {
                return None;
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert() {
        let mut index = TokenIndex::new();
        let address = Arc::new(Address::default());
        index.insert("hello world".to_string(), &address);
        assert_eq!(index.token_index.len(), 2);
        assert_eq!(index.token_index.get("hello").unwrap().len(), 1);
        assert_eq!(index.token_index.get("world").unwrap().len(), 1);
    }

    #[test]
    fn test_search() {
        let mut index = TokenIndex::new();
        let address = Arc::new(Address::default());
        index.insert("hello world".to_string(), &address);
        let result = index.search("world hello");
        assert_eq!(result.unwrap().len(), 1);
    }

    #[test]
    fn test_search_multiple() {
        let mut index = TokenIndex::new();
        let address = Arc::new(Address::default());
        index.insert("foovej 1 2 th 1000".to_string(), &address);
        index.insert("barvej 2 1001".to_string(), &address);
        index.insert("qazvej 3 1001".to_string(), &address);
        let result = index.search("2");
        assert_eq!(result.unwrap().len(), 2);
    }

    #[test]
    fn test_search_not_found() {
        let mut index = TokenIndex::new();
        let address = Arc::new(Address::default());
        index.insert("hello world".to_string(), &address);
        let result = index.search("world hello foo");
        assert_eq!(result.is_none(), true);
    }
}
