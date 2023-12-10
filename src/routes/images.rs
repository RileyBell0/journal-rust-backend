use std::io::Cursor;

use rocket::{
    http::{ContentType, Status},
    response::{self, status, Responder},
    serde::json::Json,
    Data, Request, Response, State,
};
use rocket_multipart_form_data::{
    mime, MultipartFormData, MultipartFormDataField, MultipartFormDataOptions,
};
use serde::Serialize;
use sqlx::PgPool;

use crate::db::{self, user::User};

/// For when we send back the link to a file
#[derive(Serialize)]
pub struct ImageFileLink {
    url: String,
}

/// For when an image is uploaded and we send back a success state, potentially
/// with a link to their uploaded file
#[derive(Serialize)]
pub struct ImageResponse {
    success: i32,
    file: Option<ImageFileLink>,
}

/// The data for a single image
#[derive(Debug)]
pub struct Image {
    bytes: Vec<u8>,
    data_type: ContentType,
}
impl Image {
    /// Create a new image record
    ///
    /// ### Arguments
    ///
    /// * `bytes` - the bytes that make up the image filie
    /// * `data_type` - the type of image file being stored
    pub fn new(bytes: Vec<u8>, data_type: ContentType) -> Image {
        Image { bytes, data_type }
    }
}

/// Allow us to send an image file as a response
impl<'r> Responder<'r, 'static> for Image {
    /// Sends the given image file as a response (rocket)
    fn respond_to(self, _: &'r Request<'_>) -> response::Result<'static> {
        Response::build()
            .sized_body(self.bytes.len(), Cursor::new(self.bytes))
            .header(self.data_type)
            .ok()
    }
}

/// Gets the image with the relevant ID for the given user
#[get("/<id>")]
pub async fn get(user: User, pool: &State<PgPool>, id: i32) -> Result<Image, Status> {
    // Get the image
    let mut conn = db::acquire_conn(pool.inner()).await?;
    let record = match sqlx::query!(
        "SELECT image, mime_type FROM images WHERE id = $1 AND user_id = $2",
        id,
        user.id
    )
    .fetch_one(&mut conn)
    .await
    {
        Err(_) => return Err(Status::InternalServerError),
        Ok(record) => record,
    };

    // Try and compute the mime type
    let mime_type = match ContentType::parse_flexible(&record.mime_type) {
        None => return Err(Status::InternalServerError),
        Some(mime_type) => mime_type,
    };

    Ok(Image::new(record.image, mime_type))
}

/// Stores a new image for the given user
#[post("/", data = "<data>")]
pub async fn upload(
    user: User,
    data: Data<'_>,
    content_type: &ContentType,
    pool: &State<PgPool>,
) -> status::Custom<Json<ImageResponse>> {
    let mut conn = match db::acquire_conn(pool.inner()).await {
        Err(_) => {
            return status::Custom(
                Status::InternalServerError,
                Json(ImageResponse {
                    success: 0,
                    file: None,
                }),
            )
        }
        Ok(conn) => conn,
    };

    // parse our input data as a multipart form
    let options = MultipartFormDataOptions::with_multipart_form_data_fields(vec![
        MultipartFormDataField::file("image")
            .content_type_by_string(Some(mime::IMAGE_STAR))
            .unwrap(),
    ]);
    let multipart_form_data = MultipartFormData::parse(content_type, data, options).await;
    let multipart_form_data = match multipart_form_data {
        Ok(data) => data,
        Err(_) => {
            return status::Custom(
                Status::InternalServerError,
                Json(ImageResponse {
                    success: 0,
                    file: None,
                }),
            );
        }
    };

    let photo = multipart_form_data.files.get("image"); // Use the get method to preserve file fields from moving out of the MultipartFormData instance in order to delete them automatically when the MultipartFormData instance is being dropped
    if let Some(file_fields) = photo {
        let file_field = &file_fields[0]; // Because we only put one "image" field to the allowed_fields, the max length of this file_fields is 1.
        println!("IN THE SOME SECTION");

        let _content_type = &file_field.content_type;
        let _file_name = &file_field.file_name;
        let _path = &file_field.path;

        // get the file data as a vec<u8> string, and ensure _content_type exists
        let content = rocket::tokio::fs::read(_path).await;
        if _content_type.is_none() || content.is_err() {
            return status::Custom(
                Status::InternalServerError,
                Json(ImageResponse {
                    success: 0,
                    file: None,
                }),
            );
        }
        let content = content.unwrap();
        let _content_type = _content_type.as_ref().unwrap();

        // get the mimetype and validate it's an image format
        let content_type = _content_type.essence_str();
        let mime_type = match ContentType::parse_flexible(content_type) {
            None => {
                return status::Custom(
                    Status::InternalServerError,
                    Json(ImageResponse {
                        success: 0,
                        file: None,
                    }),
                )
            }
            Some(data) => data,
        };
        if mime_type.top() != "image" {
            return status::Custom(
                Status::InternalServerError,
                Json(ImageResponse {
                    success: 0,
                    file: None,
                }),
            );
        }
        let mime_type = mime_type.to_string();
        println!("{mime_type}");

        // insert the image into the database
        let res = sqlx::query!(
            "INSERT INTO images (user_id, image, mime_type) VALUES ($1, $2, $3) RETURNING id",
            user.id,
            content,
            mime_type
        )
        .fetch_one(&mut conn)
        .await;

        // You can now deal with the uploaded file.
        if let Ok(record) = res {
            return status::Custom(
                Status::Created,
                Json(ImageResponse {
                    success: 1,
                    file: Some(ImageFileLink {
                        url: format!("https://dev.com/api/images/{}", record.id),
                    }),
                }),
            );
        }
    }

    // TODO define a maximum file size
    status::Custom(
        Status::InternalServerError,
        Json(ImageResponse {
            success: 0,
            file: None,
        }),
    )
}
