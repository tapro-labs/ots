use rocket::serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Serialize)]
pub struct Member {
    id: String,
    name: String,

    #[serde(rename(serialize = "isBot"))]
    is_bot: bool,

    deleted: bool,

    #[serde(rename(serialize = "imageUrl"))]
    image_url: String,
}

impl<'de> Deserialize<'de> for Member {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct UserObject {
            id: String,
            is_bot: bool,
            deleted: bool,
            profile: ProfileObject,
        }

        #[derive(Deserialize)]
        struct ProfileObject {
            real_name: String,
            image_512: String,
        }

        let helper = UserObject::deserialize(deserializer)?;
        Ok(Self {
            id: helper.id,
            is_bot: helper.is_bot,
            deleted: helper.deleted,
            name: helper.profile.real_name,
            image_url: helper.profile.image_512,
        })
    }
}
