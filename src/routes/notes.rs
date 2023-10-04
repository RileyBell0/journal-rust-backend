use crate::db::{user::User, Connection, Db};
use rocket::{http::Status, response::status, serde::json::Json};
use serde::{Deserialize, Serialize};

/// Represents relevant public fields of a note
#[derive(Serialize, Deserialize)]
pub struct Note {
    id: i32,
    time: i64,
    content: String,
    title: String,
}

/// Updates the content of the note associated with the given user
async fn update_note(
    user_id: i32,
    note_id: i32,
    note_content: &str,
    update_time: i64,
    title: &str,
    conn: &mut sqlx::PgConnection,
) -> bool {
    // TODO
    // Couldn't connect to the database
    // Couldn't find the row (nothing got updated)
    // success
    // result<Ok(true,false),sqlx::error>

    // Insert a new note into the database
    let res = sqlx::query!(
        "UPDATE notes SET content = $1, title = $2, update_time = $3 WHERE id = $4 AND user_id = $5 AND update_time < $6",
        note_content,
        title,
        update_time,
        note_id,
        user_id,
        update_time,
    )
    .execute(conn)
    .await;

    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn delete_note(note_id: i32, user_id: i32, conn: &mut sqlx::PgConnection) -> bool {
    let res = sqlx::query!(
        "DELETE FROM notes WHERE id = $1 AND user_id = $2",
        note_id,
        user_id
    )
    .execute(conn)
    .await;
    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn set_note_favourite(
    user_id: i32,
    note_id: i32,
    favourite: bool,
    conn: &mut sqlx::PgConnection,
) -> bool {
    // Insert a new note into the database
    let res = sqlx::query!(
        "UPDATE notes SET favourite = $1 WHERE id = $2 AND user_id = $3",
        favourite,
        note_id,
        user_id,
    )
    .execute(conn)
    .await;

    match res {
        Ok(_) => true,
        Err(_) => false,
    }
}

async fn create_note(
    user_id: i32,
    note_content: &str,
    update_time: i64,
    title: &str,
    conn: &mut sqlx::PgConnection,
) -> Option<i32> {
    // Insert a new note into the database
    let res = sqlx::query!(
        "INSERT INTO notes (user_id, content, update_time, title) VALUES ($1, $2, $3, $4) RETURNING id",
        user_id,
        note_content,
        update_time,
        title,
    )
    .fetch_one(conn)
    .await;

    match res {
        Ok(record) => Some(record.id),
        Err(_) => None,
    }
}

#[derive(Serialize)]
pub struct NoteOverview {
    id: i32,
    title: String,
    update_time: i64,
}

#[get("/notes")]
pub async fn get_all(
    mut conn: Connection<Db>,
    user: User,
) -> status::Custom<Option<Json<Vec<NoteOverview>>>> {
    let res = sqlx::query!(
        "SELECT id, title, update_time, favourite FROM notes WHERE user_id = $1 ORDER BY id",
        user.id
    )
    .fetch_all(conn.as_mut())
    .await;

    match res {
        Err(_) => status::Custom(Status::InternalServerError, None),
        Ok(records) => {
            let mut entries = Vec::with_capacity(records.len());
            for record in records {
                entries.push(NoteOverview {
                    id: record.id,
                    title: record.title,
                    update_time: record.update_time,
                });
            }

            status::Custom(Status::Ok, Some(Json(entries)))
        }
    }
}

#[get("/notes/<note_id>")]
pub async fn get_one(
    note_id: i32,
    mut conn: Connection<Db>,
    user: User,
) -> status::Custom<Option<Json<Note>>> {
    let res = sqlx::query!(
        "SELECT id, content, update_time, title FROM notes WHERE id = $1 AND user_id = $2",
        note_id,
        user.id
    )
    .fetch_one(conn.as_mut())
    .await;

    match res {
        Err(_) => status::Custom(Status::InternalServerError, None),
        Ok(record) => status::Custom(
            Status::Ok,
            Some(Json(Note {
                id: record.id,
                content: record.content,
                time: record.update_time,
                title: record.title,
            })),
        ),
    }
}

#[delete("/notes/<note_id>")]
pub async fn delete(note_id: i32, user: User, mut conn: Connection<Db>) -> Status {
    if !delete_note(note_id, user.id, conn.as_mut()).await {
        return Status::InternalServerError;
    }
    Status::Ok
}

/// Required data for updating a note
#[derive(Deserialize)]
pub struct UpdateNote {
    content: String,
    title: String,
    time: i64,
}

#[derive(Deserialize)]
pub struct SetFavourite {
    favourite: bool,
}
#[post("/notes/<note_id>", format = "json", data = "<favourite>")]
pub async fn set_favourite(
    favourite: Json<SetFavourite>,
    note_id: i32,
    mut conn: Connection<Db>,
    user: User,
) -> Status {
    if !set_note_favourite(user.id, note_id, favourite.favourite, &mut conn).await {
        return Status::InternalServerError;
    }

    Status::Ok
}

/// Updates the content of the given note
#[post("/notes/<note_id>", format = "json", data = "<note>")]
pub async fn update(
    note: Json<UpdateNote>,
    note_id: i32,
    mut conn: Connection<Db>,
    user: User,
) -> Status {
    if !update_note(
        user.id,
        note_id,
        &note.content,
        note.time,
        &note.title,
        &mut conn,
    )
    .await
    {
        return Status::InternalServerError;
    }

    Status::Ok
}

/// Returns the id of the newly created note
#[post("/notes", format = "json", data = "<note>")]
pub async fn create(
    note: Json<UpdateNote>,
    mut conn: Connection<Db>,
    user: User,
) -> status::Custom<Option<Json<i32>>> {
    let id = create_note(user.id, &note.content, note.time, &note.title, &mut conn).await;

    if let Some(id) = id {
        return status::Custom(Status::Created, Some(Json(id)));
    }
    status::Custom(Status::InternalServerError, None)
}
