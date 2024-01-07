use crate::wizard_api::CreateWizardRequest;
use poem_openapi::Object;
use sqlx::{FromRow, PgPool};
use std::{error::Error, fmt::Display};

#[derive(Debug, Object, FromRow)]
pub struct Wizard {
    pub id: Option<i32>,
    pub name: String,
    pub title: String,
    pub age: i32,
    pub image_name: Option<String>,
}

#[derive(Debug)]
pub enum WizardError {
    NotFoundError,
    SqlxError(sqlx::Error),
}

impl Display for WizardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WizardError::NotFoundError => write!(f, "Row not found!"),
            WizardError::SqlxError(err) => write!(f, "{}", err),
        }
    }
}

impl Error for WizardError {}

impl From<sqlx::Error> for WizardError {
    fn from(err: sqlx::Error) -> Self {
        WizardError::SqlxError(err)
    }
}

async fn check(id: i32, conn: &PgPool) -> bool {
    let query = "SELECT id FROM wizards WHERE id = $1";

    let result = sqlx::query(query).bind(id).fetch_one(conn).await;

    match result {
        Ok(_) => true,
        Err(_) => false,
    }
}

pub async fn create(request: CreateWizardRequest, conn: &PgPool) -> Result<Wizard, Box<dyn Error>> {
    let query = "
        INSERT INTO wizards (name, title, age)
        VALUES ($1, $2, $3)
        RETURNING id, name, title, age, image_name
    ";

    let wizard = sqlx::query_as::<_, Wizard>(query)
        .bind(request.name)
        .bind(request.title)
        .bind(request.age)
        .fetch_one(conn)
        .await?;

    Ok(wizard)
}

pub async fn get_all(conn: &PgPool) -> Result<Vec<Wizard>, Box<dyn Error>> {
    let query = "SELECT id, name, title, age, image_name FROM wizards";

    let wizards = sqlx::query_as::<_, Wizard>(query).fetch_all(conn).await?;

    Ok(wizards)
}

pub async fn get_by_id(id: i32, conn: &PgPool) -> Result<Wizard, WizardError> {
    let query = "
        SELECT id, name, title, age, image_name
        FROM wizards WHERE id = $1
    ";

    let wizard = sqlx::query_as::<_, Wizard>(query)
        .bind(id)
        .fetch_optional(conn)
        .await?;

    match wizard {
        Some(wizard) => Ok(wizard),
        None => Err(WizardError::NotFoundError),
    }
}

pub async fn update_by_id(
    id: i32,
    request: CreateWizardRequest,
    conn: &PgPool,
) -> Result<Wizard, WizardError> {
    let check = check(id, conn).await;

    match check {
        false => Err(WizardError::NotFoundError),
        true => {
            let query = "
                UPDATE wizards
                SET name = $1, title = $2, age = $3
                WHERE id = $4
                RETURNING
                id, name, title, age, image_name
            ";

            let wizard = sqlx::query_as::<_, Wizard>(query)
                .bind(request.name)
                .bind(request.title)
                .bind(request.age)
                .bind(id)
                .fetch_one(conn)
                .await?;

            Ok(wizard)
        }
    }
}

pub async fn delete_by_id(id: i32, conn: &PgPool) -> Result<(), WizardError> {
    let check = check(id, conn).await;

    match check {
        true => {
            let query = "DELETE FROM wizards WHERE id = $1";
            sqlx::query(query).bind(id).execute(conn).await?;
            Ok(())
        }
        false => Err(WizardError::NotFoundError),
    }
}
