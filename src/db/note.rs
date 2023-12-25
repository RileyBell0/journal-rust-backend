use crate::db::DbConn;
use chrono::Utc;
use serde::{Deserialize, Serialize};

/// A type-safe integer for the number of notes we're allowed to select at once
pub struct PageSize(pub i32);
pub const MAX_PAGE_SIZE: i32 = 100;
impl PageSize {
    /// Instantiate a new PageSize instance - Ensures pagesize falls into
    /// the range of (0, self::MAX_PAGE_SIZE]
    ///
    /// ### Arguments
    ///
    /// * `size` - the page size we want to use
    ///
    /// ### Returns
    ///
    /// Error on an invalid page size, or PageSize on success
    pub fn new(size: i32) -> Result<PageSize, ()> {
        if size <= 0 || size > MAX_PAGE_SIZE {
            return Err(());
        }

        Ok(PageSize(size))
    }
}

/// A note, and all the information that comes with it
#[derive(Serialize, Deserialize)]
pub struct Note {
    id: i32,
    update_time: i64,
    favourite: bool,
    title: String,
    content: String,
    is_diary: bool,
}
impl Note {
    /// Creates a new note
    ///
    /// ### Arguments
    ///
    /// * `id` - The id of the note
    /// * `title` - The title of the note
    /// * `update_time` - The timestamp of when the note was last updated
    /// * `favourite` - if the note has been favourited
    /// * `content` - The encoded string content of the note
    pub fn new(
        id: i32,
        title: String,
        update_time: i64,
        favourite: bool,
        content: String,
        is_diary: bool,
    ) -> Note {
        Note {
            id,
            title,
            update_time,
            favourite,
            content,
            is_diary,
        }
    }
}

/// The overview of a note contains all except the content.
#[derive(Serialize)]
pub struct NoteOverview {
    id: i32,
    update_time: i64,
    favourite: bool,
    title: String,
    is_diary: bool,
}
impl NoteOverview {
    /// Creates a new note overview
    ///
    /// ### Arguments
    ///
    /// * `id` - The id of the note
    /// * `title` - The title of the note
    /// * `update_time` - The timestamp of when the note was last updated
    /// * `favourite` - if the note has been favourited
    pub fn new(
        id: i32,
        title: String,
        update_time: i64,
        favourite: bool,
        is_diary: bool,
    ) -> NoteOverview {
        NoteOverview {
            id,
            title,
            update_time,
            favourite,
            is_diary,
        }
    }
}

/// We only need the ID, all other fields are optional, and we'll calculate the update_time here
#[derive(Deserialize)]
pub struct UpdateNoteInfo {
    title: Option<String>,
    content: Option<String>,
    favourite: Option<bool>,
}

/// Fields required for creating a new note. We only need the content due to
/// potential complexities for the field and its encoding
#[derive(Deserialize)]
pub struct CreateNoteInfo {
    title: Option<String>,
    content: String,
    favourite: Option<bool>,
}

/// Gets the current timestamp
fn now() -> i64 {
    Utc::now().timestamp_millis()
}

/// Grab pages of notes where is_diary is true <- the way we determine if a note
/// is just a note, or if it's also a diary entry
pub async fn get_diary_notes(
    mut conn: DbConn,
    user_id: i32,
    page: i32,
    page_size: PageSize,
) -> Result<(Vec<Note>, bool), sqlx::Error> {
    let mut records = sqlx::query!(
        "SELECT id, title, update_time, favourite, content FROM notes WHERE user_id = $1 AND is_diary = true ORDER BY id LIMIT $2 OFFSET $3",
        user_id,
        (page_size.0 + 1) as i64,
        page as i64
    )
    .fetch_all(&mut conn)
    .await?;

    // Have we hit the last result?
    let more_available = records.len() as i32 == (page_size.0 + 1);

    // Remove our buffer elem for testing if we've got more results
    if more_available {
        records.pop();
    }

    // Convert our records into note overviews
    let overviews = records
        .into_iter()
        .map(|record| {
            Note::new(
                record.id,
                record.title,
                record.update_time,
                record.favourite,
                record.content,
                true,
            )
        })
        .collect();

    Ok((overviews, more_available))
}

/// Gets the overview of the note with the requested id owned by the given user
///
/// ### Arguments
///
/// * `conn` - A connection to the database storing the note
/// * `user_id` - The id of the user that owns the note
/// * `note_id` - The id of the note itself
///
/// ### Returns
///
/// Error if we failed to contact the database, None if no note could be found
/// with the given id, or the Note's overview on success
pub async fn get_overview(
    mut conn: DbConn,
    user_id: i32,
    note_id: i32,
) -> Result<Option<NoteOverview>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT id, title, update_time, favourite, is_diary FROM notes WHERE user_id = $1 AND id = $2",
        user_id,
        note_id
    )
    .fetch_one(&mut conn)
    .await;

    // Ensure we found a note
    if let Err(sqlx::Error::RowNotFound) = record {
        return Ok(None);
    }

    // Return the note (or throw an error if we failed to get anything)
    let record = record?;
    Ok(Some(NoteOverview::new(
        record.id,
        record.title,
        record.update_time,
        record.favourite,
        record.is_diary,
    )))
}

/// Gets the requested number of note overviews at the given page
///
/// ### Arguments
///
/// * `conn` - The connection to the database in which the notes are stored
/// * `user_id` - The user id whose note overviews we should be fetching
/// * `page` - The page number we're hoping to grab note overviews from
/// * `page_size` - The max number of note overviews per page
///
/// ### Returns
///
/// Error if we failed to contact the database, otherwise a tuple of the
/// requested page of results, and a boolean of true if there's still more
/// results, or false if we've hit the end
pub async fn get_overview_many(
    mut conn: DbConn,
    user_id: i32,
    page: i32,
    page_size: PageSize,
) -> Result<(Vec<NoteOverview>, bool), sqlx::Error> {
    let mut records = sqlx::query!(
        "SELECT id, title, update_time, favourite, is_diary FROM notes WHERE user_id = $1 ORDER BY id LIMIT $2 OFFSET $3",
        user_id,
        (page_size.0 + 1) as i64,
        page as i64
    )
    .fetch_all(&mut conn)
    .await?;

    // Have we hit the last result?
    let more_available = records.len() as i32 == (page_size.0 + 1);

    // Remove our buffer elem for testing if we've got more results
    if more_available {
        records.pop();
    }

    // Convert our records into note overviews
    let overviews = records
        .into_iter()
        .map(|record| {
            NoteOverview::new(
                record.id,
                record.title,
                record.update_time,
                record.favourite,
                record.is_diary,
            )
        })
        .collect();

    Ok((overviews, more_available))
}

/// Gets the note with the requested id owned by the given user
///
/// ### Arguments
///
/// * `conn` - A connection to the database storing the note
/// * `user_id` - The id of the user that owns the note
/// * `note_id` - The id of the note itself
///
/// ### Returns
///
/// Error if we failed to contact the database, None if no note could be found
/// with the given id, or the Note on success
pub async fn get(
    mut conn: DbConn,
    user_id: i32,
    note_id: i32,
) -> Result<Option<Note>, sqlx::Error> {
    let record = sqlx::query!(
        "SELECT id, title, update_time, favourite, content, is_diary FROM notes WHERE user_id = $1 AND id = $2",
        user_id,
        note_id
    )
    .fetch_one(&mut conn)
    .await;

    // Ensure we found a note
    if let Err(sqlx::Error::RowNotFound) = record {
        return Ok(None);
    }

    // Return the note (or throw an error if we failed to get anything)
    let record = record?;
    Ok(Some(Note::new(
        record.id,
        record.title,
        record.update_time,
        record.favourite,
        record.content,
        record.is_diary,
    )))
}

/// Gets the requested number of notes at the given page
///
/// ### Arguments
///
/// * `conn` - The connection to the database in which the notes are stored
/// * `user_id` - The user id whose notes we should be fetching
/// * `page` - The page number we're hoping to grab notes from
/// * `page_size` - The max number of notes per page
///
/// ### Returns
///
/// Error if we failed to contact the database, otherwise a tuple of the
/// requested page of results, and a boolean of true if there's still more
/// results, or false if we've hit the end
pub async fn get_many(
    mut conn: DbConn,
    user_id: i32,
    page: i32,
    page_size: PageSize,
) -> Result<(Vec<Note>, bool), sqlx::Error> {
    let mut records = sqlx::query!(
        "SELECT id, title, update_time, favourite, content, is_diary FROM notes WHERE user_id = $1 ORDER BY id LIMIT $2 OFFSET $3",
        user_id,
        (page_size.0 + 1) as i64,
        page as i64
    )
    .fetch_all(&mut conn)
    .await?;

    // Have we hit the last result?
    let more_available = records.len() as i32 == (page_size.0 + 1);

    // Remove our buffer elem for testing if we've got more results
    if more_available {
        records.pop();
    }

    // Convert our records into note overviews
    let overviews = records
        .into_iter()
        .map(|record| {
            Note::new(
                record.id,
                record.title,
                record.update_time,
                record.favourite,
                record.content,
                record.is_diary,
            )
        })
        .collect();

    Ok((overviews, more_available))
}

// TODO take an optional update_time value - so that our frontend can OPTIONALLY say "Hey, here's my update, but discard this if there's something more recent"
/// Updates the content of the note associated with the given user
///
/// # Arguments
///
/// * `user_id` - The id of the user whos note we're updating
/// * `conn` - The postgres/db connection
/// * `updated_note` - The new content of the note. Fields that exist here will be updated on the note
///
/// # Returns
/// Error if we failed to contact the database, None if no note could be found to update.
/// If a note was found, return the update time we've set on success, None on failure to update
pub async fn update(
    mut conn: DbConn,
    user_id: i32,
    note_id: i32,
    update: &UpdateNoteInfo,
) -> Result<Option<Option<i64>>, sqlx::Error> {
    // Grab the current state
    let res = sqlx::query!(
        "SELECT * FROM notes WHERE user_id = $1 AND id = $2",
        user_id,
        note_id
    )
    .fetch_one(&mut conn)
    .await;
    if let Err(sqlx::Error::RowNotFound) = res {
        // No note could be found to update
        return Ok(None);
    }
    let current = res?;

    // Perform the update
    let update_time = now();
    let res = sqlx::query!(
        "UPDATE notes SET content = $1, title = $2, update_time = $3, favourite = $4 WHERE id = $5 AND user_id = $6",
        update.content.as_ref().unwrap_or_else(|| &current.content),
        update.title.as_ref().unwrap_or_else(|| &current.title),
        update_time,
        update.favourite.unwrap_or_else(|| current.favourite),
        note_id,
        user_id,
    )
    .execute(&mut conn)
    .await?;

    // Send back the update time (on success)
    if res.rows_affected() != 0 {
        Ok(Some(Some(update_time)))
    } else {
        Ok(Some(None))
    }
}

/// Deletes the note with the given id for the given user
///
/// ### Arguments
///
/// * `note_id` - The id of the note we're going to delete
/// * `user_id` - the id of the user who owns the note
/// * `conn` - a connection to the database that stores the note
///
/// ### Returns
///
/// Error if we failed to contact the database, true if the note was deleted, false
/// if we couldn't find a note to delete
pub async fn delete(note_id: i32, user_id: i32, mut conn: DbConn) -> Result<bool, sqlx::Error> {
    let res = sqlx::query!(
        "DELETE FROM notes WHERE id = $1 AND user_id = $2",
        note_id,
        user_id
    )
    .execute(&mut conn)
    .await?;

    Ok(res.rows_affected() != 0)
}

/// Creates a new note for the given user
///
/// ### Arguments
///
/// * `user_id` - The id of the user that's going to own the new note
/// * `note_content` - The encoded content of the note
///
/// ### Returns
///
/// The id of the created note on succes, or an sqlx::Error on failure
pub async fn create(
    mut conn: DbConn,
    user_id: i32,
    note: &CreateNoteInfo,
) -> Result<Note, sqlx::Error> {
    // Insert a new note into the database
    let record = sqlx::query!(
        "INSERT INTO notes (user_id, content, update_time, title, favourite) VALUES ($1, $2, $3, $4, $5) RETURNING *",
        user_id,
        note.content,
        now(),
        note.title.as_deref().unwrap_or(""),
        note.favourite.unwrap_or(false)
    )
    .fetch_one(&mut conn)
    .await?; // if fetch_one fails, something went wrong internally and the note wasn't created

    Ok(Note::new(
        record.id,
        record.title,
        record.update_time,
        record.favourite,
        record.content,
        record.is_diary,
    ))
}
