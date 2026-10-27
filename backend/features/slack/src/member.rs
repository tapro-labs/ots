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

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::serde::json::{json, serde_json, Value};

    // Trimmed copy of a member from Slack's `users.list` response.
    fn slack_user() -> Value {
        json!({
            "id": "U023BECGF",
            "team_id": "T021F9ZE2",
            "name": "bobby",
            "deleted": false,
            "real_name": "Bobby Tables",
            "is_admin": true,
            "is_bot": false,
            "updated": 1502138686,
            "profile": {
                "real_name": "Bobby Tables",
                "display_name": "bobby",
                "image_72": "https://example.com/72.jpg",
                "image_512": "https://example.com/512.jpg"
            }
        })
    }

    #[test]
    fn test_deserializes_slack_user_object() {
        let member: Member = serde_json::from_value(slack_user()).unwrap();

        assert_eq!(member.id, "U023BECGF");
        // The name comes from the profile real name, not the user handle.
        assert_eq!(member.name, "Bobby Tables");
        assert!(!member.is_bot);
        assert!(!member.deleted);
        assert_eq!(member.image_url, "https://example.com/512.jpg");
    }

    #[test]
    fn test_deserializes_bot_and_deleted_flags() {
        let mut user = slack_user();
        user["is_bot"] = json!(true);
        user["deleted"] = json!(true);

        let member: Member = serde_json::from_value(user).unwrap();

        assert!(member.is_bot);
        assert!(member.deleted);
    }

    #[test]
    fn test_deserialize_fails_without_required_profile_fields() {
        let mut without_image = slack_user();
        without_image["profile"]
            .as_object_mut()
            .unwrap()
            .remove("image_512");
        let mut without_profile = slack_user();
        without_profile.as_object_mut().unwrap().remove("profile");

        assert!(serde_json::from_value::<Member>(without_image).is_err());
        assert!(serde_json::from_value::<Member>(without_profile).is_err());
    }

    #[test]
    fn test_serializes_with_frontend_field_names() {
        let member: Member = serde_json::from_value(slack_user()).unwrap();

        assert_eq!(
            serde_json::to_value(&member).unwrap(),
            json!({
                "id": "U023BECGF",
                "name": "Bobby Tables",
                "isBot": false,
                "deleted": false,
                "imageUrl": "https://example.com/512.jpg"
            })
        );
    }
}
