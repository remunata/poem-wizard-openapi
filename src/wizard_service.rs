use crate::wizard_api::CreateWizardRequest;
use poem_openapi::{types::multipart::Upload, Object};
use sqlx::{FromRow, PgPool};
use std::{
    error::Error,
    ffi::OsStr,
    fmt::Display,
    fs,
    io::Write,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

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
    ExtError(Box<dyn std::error::Error>),
}

impl Display for WizardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WizardError::NotFoundError => write!(f, "Row not found!"),
            WizardError::ExtError(err) => write!(f, "{}", err),
        }
    }
}

impl Error for WizardError {}

impl From<sqlx::Error> for WizardError {
    fn from(err: sqlx::Error) -> Self {
        WizardError::ExtError(Box::new(err))
    }
}

impl From<std::io::Error> for WizardError {
    fn from(err: std::io::Error) -> Self {
        WizardError::ExtError(Box::new(err))
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

    tracing::info!("New wizard created: {:?}", &wizard);

    Ok(wizard)
}

#[tracing::instrument]
pub async fn get_all(conn: &PgPool) -> Result<Vec<Wizard>, Box<dyn Error>> {
    let query = "
        SELECT id, name, title, age, image_name 
        FROM wizards
        ORDER BY id ASC
    ";

    let wizards = sqlx::query_as::<_, Wizard>(query).fetch_all(conn).await?;

    tracing::info!("Fetch all wizards data");

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

            tracing::info!("Wizard updated: {:?}", &wizard);

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
            tracing::info!("Wizard with id {} deleted", id);
            Ok(())
        }
        false => Err(WizardError::NotFoundError),
    }
}

pub async fn save_image(id: i32, conn: &PgPool, request: Upload) -> Result<String, WizardError> {
    let query = "
        SELECT id, name, title, age, image_name
        FROM wizards WHERE id = $1
    ";

    let wizard = sqlx::query_as::<_, Wizard>(query)
        .bind(id)
        .fetch_optional(conn)
        .await?;

    match wizard {
        None => Err(WizardError::NotFoundError),
        Some(wizard) => {
            if let Some(img_name) = wizard.image_name {
                let img_path = format!("./files/{}", img_name);
                let result = fs::remove_file(&img_path);
                if let Ok(_) = result {
                    tracing::info!("File {} deleted", img_path);
                }
            }

            let filename = request.file_name().unwrap_or_else(|| "default.png");
            let ext = Path::new(filename).extension().and_then(OsStr::to_str);
            let ext = ext.unwrap_or_else(|| "png");

            let milis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis();

            let filename = format!("{}{}.{}", milis, id, ext);
            let path = format!("./files/{}", &filename);

            let data = request.into_vec().await?;
            let mut file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .open(&path)?;
            let _ = file.write_all(&data);

            tracing::info!("File {} stored in server", path);

            let query = "UPDATE wizards SET image_name = $1 WHERE id = $2";
            sqlx::query(query)
                .bind(&filename)
                .bind(id)
                .execute(conn)
                .await?;

            tracing::info!(
                "Wizard image updated, id: {}, image_name: {}",
                id,
                &filename
            );

            Ok(filename)
        }
    }
}
