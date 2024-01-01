use poem::{error::{InternalServerError, NotFound}, web::Data, Result};
use poem_openapi::{
    param::Query,
    payload::{Json, PlainText},
    OpenApi,
};
use sqlx::PgPool;

use crate::wizard::{CreateWizard, Wizard};

pub struct WizardApi;

#[OpenApi]
impl WizardApi {
    #[oai(path = "/", method = "get")]
    async fn index(&self, name: Query<Option<String>>) -> PlainText<String> {
        match name.0 {
            Some(name) => PlainText(format!("Hello, {}!", name)),
            None => PlainText("Hello, wizard!".to_string()),
        }
    }
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
        .map_err(InternalServerError)?
        .id;

        Ok(Json(id))
    }
    #[oai(path = "/wizards", method = "get")]
    async fn get_all(&self, pool: Data<&PgPool>) -> Result<Json<Vec<Wizard>>> {
        let wizards = sqlx::query_as!(Wizard, "SELECT * FROM wizards")
            .fetch_all(pool.0)
            .await
            .unwrap();

        Ok(Json(wizards))
    }
    #[oai(path = "/wizards/:id", method = "get")]
    async fn get_by_id(&self, pool: Data<&PgPool>, id: Query<i32>) -> Result<Json<Wizard>> {
        let wizard = sqlx::query_as!(
            Wizard,
            r#"SELECT * FROM wizards WHERE id = $1"#,
            id.0
        )
        .fetch_one(pool.0)
        .await
        .map_err(NotFound)?;

        Ok(Json(wizard))
    }
}
