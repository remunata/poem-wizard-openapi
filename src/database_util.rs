use sqlx::PgPool;

pub async fn create_wizard_table(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS wizards(
            id SERIAL PRIMARY KEY,
            name VARCHAR NOT NULL,
            title VARCHAR NOT NULL,
            age INT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}
