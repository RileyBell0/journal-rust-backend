use crate::db;

#[get("/<id>")]
pub async fn get_user(id: i32, mut db: db::Connection<db::Db>) -> Option<String> {
    let rows = rocket_db_pools::sqlx::query!("SELECT * FROM account where id = $1", id)
        .fetch_all(&mut *db)
        .await;

    let rows = match rows {
        Ok(r) => r,
        _ => return Some("Failed".to_string()),
    };

    let mut info = Vec::new();

    for row in rows {
        info.push(format!("({},{})", row.id, row.name));
    }

    Some(info.join("\n"))
}
