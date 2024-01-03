use poem::{
    error::{BadRequest, NotFound},
    web::Data,
    Result,
};
use poem_openapi::{
    param::Path,
    payload::{Attachment, AttachmentType, Json, PlainText},
    types::multipart::Upload,
    ApiResponse, Multipart, Object, OpenApi,
};
use sqlx::PgPool;

#[derive(Debug, Object)]
struct Wizard {
    id: i32,
    name: String,
    title: String,
    age: i32,
}

#[derive(Debug, Object)]
struct CreateWizard {
    name: String,
    title: String,
    age: i32,
}

#[derive(Debug, Object, Clone)]
struct Image {
    image_name: Option<String>,
    image: Option<Vec<u8>>,
}

#[derive(Debug, Multipart)]
struct UploadPayload {
    file: Upload,
}

#[derive(Debug, ApiResponse)]
enum GetImageResponse {
    #[oai(status = 200)]
    Ok(Attachment<Vec<u8>>),
    #[oai(status = 404)]
    NotFound,
}

pub struct WizardApi;

#[OpenApi]
impl WizardApi {
    /// Add New Wizard
    #[oai(path = "/wizards", method = "post")]
    async fn add(&self, pool: Data<&PgPool>, wizard: Json<CreateWizard>) -> Result<Json<i32>> {
        let id: i32 = sqlx::query!(
            "INSERT INTO wizards (name, title, age) VALUES ($1, $2, $3) RETURNING id",
            wizard.name,
            wizard.title,
            wizard.age
        )
        .fetch_one(pool.0)
        .await
        .map_err(BadRequest)?
        .id;

        Ok(Json(id))
    }

    /// Get All Wizards
    #[oai(path = "/wizards", method = "get")]
    async fn get_all(&self, pool: Data<&PgPool>) -> Result<Json<Vec<Wizard>>> {
        let wizards = sqlx::query_as!(Wizard, "SELECT id, name, title, age FROM wizards")
            .fetch_all(pool.0)
            .await
            .unwrap();

        Ok(Json(wizards))
    }

    /// Get Wizard by Id
    #[oai(path = "/wizards/:id", method = "get")]
    async fn get_by_id(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<Json<Wizard>> {
        let wizard = sqlx::query_as!(
            Wizard,
            r#"SELECT id, name, title, age FROM wizards WHERE id = $1"#,
            id.0
        )
        .fetch_one(pool.0)
        .await
        .map_err(NotFound)?;

        Ok(Json(wizard))
    }

    /// Update Wizard By Id
    #[oai(path = "/wizards/:id", method = "put")]
    async fn update(
        &self,
        pool: Data<&PgPool>,
        id: Path<i32>,
        wizard: Json<CreateWizard>,
    ) -> Result<Json<Wizard>> {
        let wizard = sqlx::query_as!(
            Wizard,
            r#"UPDATE wizards SET name = $1, title = $2, age = $3 WHERE id = $4 RETURNING id, name, title, age"#,
            wizard.name,
            wizard.title,
            wizard.age,
            id.0
        )
        .fetch_one(pool.0)
        .await
        .map_err(NotFound)?;

        Ok(Json(wizard))
    }

    /// Delete Wizard by Id
    #[oai(path = "/wizards/:id", method = "delete")]
    async fn delete(&self, pool: Data<&PgPool>, id: Path<i32>) -> PlainText<String> {
        sqlx::query!(r#"DELETE FROM wizards WHERE id = $1"#, id.0)
            .execute(pool.0)
            .await
            .unwrap();

        PlainText(format!("Wizard with id {} deleted", id.0))
    }

    /// Upload Wizard Image by Id
    #[oai(path = "/wizards/:id/image", method = "post")]
    async fn upload_image(
        &self,
        pool: Data<&PgPool>,
        id: Path<i32>,
        upload: UploadPayload,
    ) -> Result<PlainText<String>> {
        let filename = upload.file.file_name().map(ToString::to_string);
        let data = upload.file.into_vec().await.map_err(BadRequest)?;

        sqlx::query!(
            r#"UPDATE wizards SET image_name = $1, image = $2 WHERE id = $3"#,
            filename,
            data,
            id.0
        )
        .execute(pool.0)
        .await
        .map_err(BadRequest)?;

        Ok(PlainText("Upload Image Success".to_string()))
    }

    /// Get Wizard Image by Id
    #[oai(path = "/wizards/:id/image", method = "get")]
    async fn get_image(&self, pool: Data<&PgPool>, id: Path<i32>) -> Result<GetImageResponse> {
        let file = sqlx::query_as!(
            Image,
            r#"SELECT image_name, image FROM wizards WHERE id = $1"#,
            id.0
        )
        .fetch_one(pool.0)
        .await
        .map_err(NotFound)?;

        match file.image {
            Some(image) => {
                let mut attachment =
                    Attachment::new(image.clone()).attachment_type(AttachmentType::Attachment);
                if let Some(filename) = &file.image_name {
                    attachment = attachment.filename(filename);
                }
                Ok(GetImageResponse::Ok(attachment))
            }
            None => Ok(GetImageResponse::NotFound),
        }
    }
}
