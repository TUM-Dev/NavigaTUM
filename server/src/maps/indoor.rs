use sqlx::{PgPool, Row};
#[tracing::instrument(skip(pool))]
pub async fn list_indoor_inside_of(pool:&PgPool) ->anyhow::Result<Vec<i64>>{
    let geom: geo_types::Geometry<f64> = geo::Point::new(10.0, 20.0).into();
    let filtered_groups = sqlx::query("SELECT group_id from indoor_features where ST_Contains(convex_hull::geometry, $1::geometry)")
        .bind(geozero::wkb::Encode(geom))
        .fetch_all(pool)
        .await?;
    let mut filtered_group_ids =Vec::<i64>::new();
    for group in filtered_groups {
        let group_id = group.get_unchecked(0);
        filtered_group_ids.push(group_id);
    }

    Ok(filtered_group_ids)
}