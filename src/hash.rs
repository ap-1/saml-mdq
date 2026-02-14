use sha1::{Digest, Sha1};

/// Convert an entityID to its SHA-1 hex hash for MDQ lookup.
pub fn hash_entity_id(entity_id: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(entity_id.as_bytes());
    hex::encode(hasher.finalize())
}

/// URL-encode an entityID for MDQ lookup.
pub fn encode_entity_id(entity_id: &str) -> String {
    url::form_urlencoded::byte_serialize(entity_id.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_produces_40_hex_chars() {
        let hash = hash_entity_id("https://idp.example.org/shibboleth");
        assert_eq!(hash.len(), 40);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn hash_is_deterministic() {
        let a = hash_entity_id("https://idp.example.org/shibboleth");
        let b = hash_entity_id("https://idp.example.org/shibboleth");
        assert_eq!(a, b);
    }

    #[test]
    fn encode_percent_encodes_url() {
        let encoded = encode_entity_id("https://idp.example.org/shibboleth");
        assert_eq!(encoded, "https%3A%2F%2Fidp.example.org%2Fshibboleth");
    }
}
