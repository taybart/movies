use anyhow::Result;
use serde::Serialize;
use sqlx::{
    prelude::FromRow, sqlite::SqliteRow, Pool, QueryBuilder, Row, Sqlite, SqlitePool, Transaction,
};

#[derive(Debug, Serialize)]
pub struct Crew {
    pub tconst: String,
    pub directors: Vec<String>,
    pub writers: Vec<String>,
}

pub async fn init_table(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS crew (
                tconst TEXT PRIMARY KEY,
                directors TEXT,
                writers TEXT
            )",
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub struct CrewQuery<'a>(QueryBuilder<'a, Sqlite>);

impl<'r> FromRow<'r, SqliteRow> for Crew {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let writers_str = row.try_get("writers").unwrap_or("".to_owned());
        let writers = writers_str.split(',').map(|s| s.to_owned()).collect();
        let directors_str = row.try_get("directors").unwrap_or("".to_owned());
        let directors = directors_str.split(',').map(|s| s.to_owned()).collect();

        Ok(Self {
            tconst: row.try_get("tconst").unwrap_or("".into()),
            writers,
            directors,
        })
    }
}

impl<'a> CrewQuery<'a> {
    pub fn new() -> Self {
        CrewQuery(QueryBuilder::new("SELECT * FROM crew"))
    }

    pub fn id(mut self, id: &'a String) -> Self {
        if !id.is_empty() {
            self.where_and();
            self.0.push(" tconst = ");
            self.0.push_bind(id);
        }
        self
    }

    fn where_and(&mut self) {
        if !self.0.sql().contains("WHERE") {
            self.0.push(" WHERE");
        } else {
            self.0.push(" AND");
        }
    }

    pub async fn fetch_one(mut self, db: &SqlitePool) -> Result<Crew> {
        Ok(self.0.build_query_as::<Crew>().fetch_one(db).await?)
    }
}

pub async fn ingest(
    transaction: &mut Transaction<'_, Sqlite>,
    record: &Vec<String>,
) -> Result<(), sqlx::Error> {
    if record.len() >= 3 {
        sqlx::query(
            "INSERT OR IGNORE INTO crew 
                (tconst, directors, writers)
                VALUES (?, ?, ?)",
        )
        .bind(&record[0])
        .bind(&record[1])
        .bind(&record[2])
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}
