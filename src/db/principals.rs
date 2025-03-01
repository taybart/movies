use anyhow::Result;
use serde::Serialize;
use sqlx::{
    prelude::FromRow, sqlite::SqliteRow, Pool, QueryBuilder, Row, Sqlite, SqlitePool, Transaction,
};

#[derive(Debug, Serialize)]
pub struct Principal {
    pub tconst: String,
    pub ordering: i64,
    pub nconst: String,
    pub category: String,
    pub job: String,
    // TODO: next ingest make this column an array
    pub characters: Vec<String>,
}

pub async fn init_table(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS principals (
                tconst TEXT,
                ordering INTEGER,
                nconst TEXT,
                category TEXT,
                job TEXT,
                characters TEXT,
                PRIMARY KEY (tconst, ordering)
            )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub struct PrincipalsQuery<'a>(QueryBuilder<'a, Sqlite>);

impl<'r> FromRow<'r, SqliteRow> for Principal {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let characters_str = row.try_get("writers").unwrap_or("".to_owned());
        let characters = characters_str.split(',').map(|s| s.to_owned()).collect();

        Ok(Self {
            tconst: row.try_get("tconst").unwrap_or("".into()),
            ordering: row.try_get("ordering").unwrap_or(0),
            nconst: row.try_get("nconst").unwrap_or("".into()),
            category: row.try_get("category").unwrap_or("".into()),
            job: row.try_get("job").unwrap_or("".into()),
            characters,
        })
    }
}

impl<'a> PrincipalsQuery<'a> {
    pub fn new() -> Self {
        PrincipalsQuery(QueryBuilder::new("SELECT * FROM principals"))
    }

    pub fn movie(mut self, id: &'a String) -> Self {
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

    pub async fn fetch_one(mut self, db: &SqlitePool) -> Result<Principal> {
        Ok(self.0.build_query_as::<Principal>().fetch_one(db).await?)
    }
    pub async fn fetch(mut self, db: &SqlitePool) -> Result<Vec<Principal>> {
        Ok(self.0.build_query_as::<Principal>().fetch_all(db).await?)
    }
}

pub async fn ingest(
    transaction: &mut Transaction<'_, Sqlite>,
    record: &Vec<String>,
) -> Result<(), sqlx::Error> {
    if record.len() >= 6 {
        sqlx::query(
            r#"INSERT OR IGNORE INTO principals 
                (tconst, ordering, nconst, category, job, characters)
                VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&record[0])
        .bind(record[1].parse::<i32>().unwrap_or(0))
        .bind(&record[2])
        .bind(&record[3])
        .bind(&record[4])
        .bind(&record[5])
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}
