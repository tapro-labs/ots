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
