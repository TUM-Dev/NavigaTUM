use sqlx::PgPool;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;

#[allow(dead_code)] // used for testing out the repo pattern
pub struct RoomSection {
    pub id: String,
    pub name: String,
}
impl RoomSection {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool, id: &str) -> sqlx::Result<HashMap<String, Vec<Self>>> {
        struct RoomSectionQuery {
            usage_name: Option<String>,
            location_id: Option<String>,
            location_name: Option<String>,
        }
        let entries: Vec<RoomSectionQuery> = sqlx::query_as!(
            RoomSectionQuery,
            r#"Select usage_name,location_id,location_name
            from rooms_section
            where key=$1"#,
            id
        )
        .fetch_all(pool)
        .await?;
        let mut result = HashMap::<String, Vec<RoomSection>>::new();
        for r in entries.into_iter() {
            let entry = RoomSection {
                id: r.location_id.expect("sqlx bug, cannot be none"),
                name: r.location_name.expect("sqlx bug, cannot be none"),
            };
            match result.entry(r.usage_name.expect("sqlx bug, cannot be none")) {
                Occupied(mut usage) => {
                    usage.get_mut().push(entry);
                }
                Vacant(usage) => {
                    usage.insert_entry(vec![entry]);
                }
            }
        }

        Ok(result)
    }
}

#[allow(dead_code)] // used for testing out the repo pattern
pub struct BuildingSection {
    pub id: Option<String>,
    pub name: Option<String>,
    pub thumb: Option<String>,
    pub subtext: Option<String>,
    pub visible: Option<bool>,
}
impl BuildingSection {
    #[tracing::instrument(skip(pool))]
    pub async fn fetch_all(pool: &PgPool, id: &str) -> sqlx::Result<Vec<Self>> {
        sqlx::query_as!(
            BuildingSection,
            r#"Select id,name,thumb,subtext,visible
            from buildings_section
            where key=$1"#,
            id
        )
        .fetch_all(pool)
        .await
    }
}
