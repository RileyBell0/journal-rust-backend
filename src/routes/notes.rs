use crate::db::{
    self,
    note::{self, CreateNoteInfo, Note, NoteOverview, UpdateNoteInfo},
    user::User,
};
use rocket::{http::Status, response::status, serde::json::Json, State};
use serde::Serialize;
use sqlx::PgPool;

/// Represents a generic paged response - the data and if there's more after this
#[derive(Serialize)]
pub struct PagedResponse<T> {
    data: T,
    more: bool,
}

/// Creates a new note, returning the ID of the new note
///
/// ### Arguments
///
/// * `create` - the information required to create the note
/// * `pool` - a pool of connections to the database we want to create the note in
/// * `user` - the user creating the note
#[post("/", format = "json", data = "<create>")]
pub async fn create(
    create: Json<CreateNoteInfo>,
    pool: &State<PgPool>,
    user: User,
) -> status::Custom<Option<Json<i32>>> {
    let conn = match db::acquire_conn(pool).await {
        Ok(conn) => conn,
        Err(_) => return status::Custom(Status::InternalServerError, None),
    };

    // Create the note, returning the ID of the created note on success, or an error on failure
    match note::create(conn, user.id, &create).await {
        Err(_) => status::Custom(Status::InternalServerError, None),
        Ok(id) => status::Custom(Status::Created, Some(Json(id))),
    }
}

/// Gets the note with the specified ID
///
/// ### Arguments
///
/// * `note_id` - the id of the note we're wanting to fetch
/// * `pool` - connections to the database where our note is stored
/// * `user` - the user that's making the request
///
/// ### Returns
///
/// * `Status::InternalServerError` if we couldn't get the note, or we failed to contact the database
/// * `Status::NotFound` if no such note with the given id exists for the user
/// * `Status::Ok` if we got the note, and the json encoded note itself
#[get("/<note_id>")]
pub async fn get(
    note_id: i32,
    pool: &State<PgPool>,
    user: User,
) -> status::Custom<Option<Json<Note>>> {
    let conn = match db::acquire_conn(pool).await {
        Ok(conn) => conn,
        Err(_) => return status::Custom(Status::InternalServerError, None),
    };

    // attempt to get the note, and return
    match note::get(conn, user.id, note_id).await {
        Ok(Some(note)) => status::Custom(Status::Ok, Some(Json(note))),
        Ok(None) => status::Custom(Status::NotFound, None),
        Err(_) => status::Custom(Status::InternalServerError, None),
    }
}

/// Gets the data for multiple notes at once, batched in sizes of page_size
///
/// ### Arguments
///
/// * `pool` - connections to the db that's storing our notes
/// * `user` - the user who's making the requeset
/// * `page` - the numbered page we're hoping to get data for
/// * `page_size` - how many results in each page
///
/// ### Returns
///
/// * `status::InternalServerError` when we failed to reach thedb, or couldn't get the notes
/// * `status::BadRequest` if an invalid pagesize was returned
/// * `status::Ok` and a json-encoded vector of notes, and a bool for if there's more results on success
#[get("/?<page>&<page_size>")]
pub async fn get_many(
    pool: &State<PgPool>,
    user: User,
    page: i32,
    page_size: Option<i32>,
) -> status::Custom<Option<Json<PagedResponse<Vec<Note>>>>> {
    let conn = match db::acquire_conn(pool).await {
        Ok(conn) => conn,
        Err(_) => return status::Custom(Status::InternalServerError, None),
    };

    // Validate input parameter
    let page_size = match note::PageSize::new(page_size.unwrap_or(20)) {
        Ok(page_size) => page_size,
        Err(_) => return status::Custom(Status::BadRequest, None),
    };

    // Fetch and return
    match note::get_many(conn, user.id, page, page_size).await {
        Ok(notes) => status::Custom(
            Status::Ok,
            Some(Json(PagedResponse {
                data: notes.0,
                more: notes.1,
            })),
        ),
        Err(_) => status::Custom(Status::InternalServerError, None),
    }
}

/// Gets the overview for the note with the specified id
///
/// ### Arguments
///
/// * `note_id` - the ID of the note we're retrieving
/// * `pool` - a pool of connections to the database where the note is located
/// * `user` - the user making the request / the user that owns the note
///
/// ### Returns
///
/// `Status::InternalServerError` if we failed to contact the database
/// `Status::NotFound` if no such note exists for the given user
/// `Status::Ok` if we found the note, with the note overview attached (and json encoded)
#[get("/<note_id>/overview")]
pub async fn get_overview(
    note_id: i32,
    pool: &State<PgPool>,
    user: User,
) -> status::Custom<Option<Json<NoteOverview>>> {
    let conn = match db::acquire_conn(pool).await {
        Ok(conn) => conn,
        Err(_) => return status::Custom(Status::InternalServerError, None),
    };

    // Grab the overview, or throw the relevant error on failure
    match note::get_overview(conn, user.id, note_id).await {
        Ok(Some(note_overview)) => status::Custom(Status::Ok, Some(Json(note_overview))),
        Ok(None) => status::Custom(Status::NotFound, None),
        Err(_) => status::Custom(Status::InternalServerError, None),
    }
}

/// Gets multiple note overviews, batched in sizes of page_size
///
/// ### Arguments
///
/// * `pool` - connections to the db that's storing our notes
/// * `user` - the user who's making the requeset
/// * `page` - the numbered page we're hoping to get data for
/// * `page_size` - how many results in each page
///
/// ### Returns
///
/// * `status::InternalServerError` when we failed to reach thedb, or couldn't get the notes
/// * `status::BadRequest` if an invalid pagesize was returned
/// * `status::Ok` and a json-encoded vector of notes, and a bool for if there's more results on success
#[get("/?<page>&<page_size>&overview=true")]
pub async fn get_overview_many(
    pool: &State<PgPool>,
    user: User,
    page: i32,
    page_size: Option<i32>,
) -> status::Custom<Option<Json<PagedResponse<Vec<NoteOverview>>>>> {
    let conn = match db::acquire_conn(pool).await {
        Ok(conn) => conn,
        Err(_) => return status::Custom(Status::InternalServerError, None),
    };

    // Validate input parameter
    let page_size = match note::PageSize::new(page_size.unwrap_or(20)) {
        Ok(page_size) => page_size,
        Err(_) => return status::Custom(Status::BadRequest, None),
    };

    // Fetch and return
    match note::get_overview_many(conn, user.id, page, page_size).await {
        Ok(notes) => status::Custom(
            Status::Ok,
            Some(Json(PagedResponse {
                data: notes.0,
                more: notes.1,
            })),
        ),
        Err(_) => status::Custom(Status::InternalServerError, None),
    }
}

/// Delete the note with the given id owned by the provided user
///
/// ### Arguments
///
/// * `note_id` - the ID of the note to be deleted
/// * `pool` - a pool of connections to the database where the note is stored
/// * `user` - the user who owns the note / is executing the request
///
/// ### Returns
///
/// * `Status::InternalServerError` if we failed to contact the database
/// * `Status::NotFound` if no such note could be found
/// * `Status::Ok` if the note was successfully deleted
#[delete("/<note_id>")]
pub async fn delete(note_id: i32, pool: &State<PgPool>, user: User) -> Status {
    let conn = match db::acquire_conn(pool).await {
        Ok(conn) => conn,
        Err(_) => return Status::InternalServerError,
    };

    // Attempt to delete the specified note, then return the status of said deletion
    match note::delete(note_id, user.id, conn).await {
        Err(_) => Status::InternalServerError,
        Ok(false) => Status::NotFound,
        Ok(true) => Status::Ok,
    }
}

/// Update the given note
///
/// ### Arguments
///
/// * `note_id` - the id of the note we're updating
/// * `update` - the update package, containing only the fields we're hoping to update
/// * `pool` - a pool of connections to the database in which the note is stored
/// * `user` - the user who owns the note / the user who's making the request
#[patch("/<note_id>", format = "json", data = "<update>")]
pub async fn update(
    note_id: i32,
    update: Json<UpdateNoteInfo>,
    pool: &State<PgPool>,
    user: User,
) -> Status {
    let conn = match db::acquire_conn(pool).await {
        Ok(conn) => conn,
        Err(_) => return Status::InternalServerError,
    };

    // Perform the update
    match note::update(conn, user.id, note_id, &update).await {
        Err(_) => Status::InternalServerError, // failed to talk to the db
        Ok(Some(false)) => Status::InternalServerError, // failed to update
        Ok(None) => Status::NotFound,          // no such note exists
        Ok(Some(true)) => Status::Ok,          // success
    }
}
