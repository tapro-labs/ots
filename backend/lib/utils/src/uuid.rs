use std::fmt::{Display, Formatter};

use rocket::serde::{Serialize, Serializer};
use uuid::Uuid as ExternalUuid;

pub struct Uuid {
    value: String,
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.value.as_str())
    }
}

impl Uuid {
    pub fn new_v4() -> Self {
        Self {
            value: ExternalUuid::new_v4().to_string(),
        }
    }
}

impl Display for Uuid {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    #[test]
    fn test_new_v4_is_a_valid_hyphenated_v4_uuid() {
        let value = Uuid::new_v4().to_string();
        let parsed = ExternalUuid::parse_str(&value).expect("valid uuid");

        assert_eq!(parsed.get_version_num(), 4);
        assert_eq!(value.len(), 36);
        assert_eq!(value, parsed.hyphenated().to_string());
    }

    #[test]
    fn test_new_v4_values_are_unique() {
        let values: HashSet<String> = (0..1000).map(|_| Uuid::new_v4().to_string()).collect();

        assert_eq!(values.len(), 1000);
    }

    #[test]
    fn test_serializes_as_plain_json_string() {
        let uuid = Uuid::new_v4();
        let json = rocket::serde::json::to_string(&uuid).unwrap();

        assert_eq!(json, format!("\"{}\"", uuid));
    }
}
