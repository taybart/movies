use sqlx::{Pool, Sqlite, Transaction};

pub async fn init_table(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS episodes (
                tconst TEXT PRIMARY KEY,
                parentTconst TEXT,
                seasonNumber INTEGER,
                episodeNumber INTEGER
            )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn ingest(
    transaction: &mut Transaction<'_, Sqlite>,
    record: &Vec<String>,
) -> Result<(), sqlx::Error> {
    if record.len() >= 4 {
        sqlx::query(
            "INSERT OR IGNORE INTO episodes 
                            (tconst, parentTconst, seasonNumber, episodeNumber)
                            VALUES (?, ?, ?, ?)",
        )
        .bind(&record[0])
        .bind(&record[1])
        .bind(if record[2] == "\\N" {
            None
        } else {
            record[2].parse::<i32>().ok()
        })
        .bind(if record[3] == "\\N" {
            None
        } else {
            record[3].parse::<i32>().ok()
        })
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}
