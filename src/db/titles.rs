use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::{
    query_builder::QueryBuilder, sqlite::SqliteRow, FromRow, Pool, Row, Sqlite, SqlitePool,
    Transaction,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Title {
    pub tconst: String,
    pub title_type: String,
    pub primary_title: String,
    pub original_title: String,
    pub is_adult: i64,
    pub start_year: i64,
    pub end_year: i64,
    pub runtime_minutes: i64,
    pub genres: String,
}

pub struct TitleQuery<'a>(QueryBuilder<'a, Sqlite>);

impl<'r> FromRow<'r, SqliteRow> for Title {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            tconst: row.try_get("tconst").unwrap_or("".into()),
            title_type: row.try_get("title_type").unwrap_or("".into()),
            primary_title: row.try_get("primary_title").unwrap_or("".into()),
            original_title: row.try_get("original_title").unwrap_or("".into()),
            is_adult: row.try_get("is_adult").unwrap_or(0),
            start_year: row.try_get("start_year").unwrap_or(0),
            end_year: row.try_get("end_year").unwrap_or(0),
            runtime_minutes: row.try_get("runtime_minutes").unwrap_or(0),
            genres: row.try_get("genres").unwrap_or("".into()),
        })
    }
}

pub async fn init_table(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS titles (
                tconst TEXT PRIMARY KEY NOT NULL,
                title_type TEXT,
                primary_title TEXT,
                original_title TEXT,
                is_adult INTEGER,
                start_year INTEGER,
                end_year INTEGER,
                runtime_minutes INTEGER,
                genres TEXT
            )"#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS title_akas (
                title_id TEXT,
                ordering INTEGER,
                title TEXT,
                region TEXT,
                language TEXT,
                types TEXT,
                attributes TEXT,
                is_original_title INTEGER,
                PRIMARY KEY (title_id, ordering)
            )
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

impl<'a> TitleQuery<'a> {
    pub fn new() -> Self {
        TitleQuery(QueryBuilder::new("SELECT * FROM titles"))
    }

    pub fn id(mut self, id: &'a String) -> Self {
        if !id.is_empty() {
            self.where_and();
            self.0.push(" tconst = ");
            self.0.push_bind(id);
        }
        self
    }

    pub fn like(mut self, title: String) -> Self {
        if !title.is_empty() {
            self.where_and();
            self.0.push(" original_title LIKE ");
            self.0.push_bind(format!("{}%", title));
            self.0.push(" COLLATE NOCASE ");
        }
        self
    }

    pub fn title_type(mut self, title_type: String) -> Self {
        if !title_type.is_empty() {
            self.where_and();
            self.0.push(" title_type = ");
            self.0.push_bind(title_type);
        }
        self
    }

    pub fn start_year(mut self, year: Option<i64>) -> Self {
        if year.is_some() {
            self.where_and();
            self.0.push(" start_year = ");
            self.0.push_bind(year.unwrap());
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

    pub async fn fetch_one(mut self, db: &SqlitePool) -> Result<Title> {
        Ok(self.0.build_query_as::<Title>().fetch_one(db).await?)
    }
    pub async fn fetch(mut self, db: &SqlitePool) -> Result<Vec<Title>> {
        Ok(self.0.build_query_as::<Title>().fetch_all(db).await?)
    }
}

pub async fn ingest(
    transaction: &mut Transaction<'_, Sqlite>,
    record: &Vec<String>,
) -> Result<(), sqlx::Error> {
    if record.len() >= 9 {
        let is_adult = record[4].parse::<i32>().unwrap_or(0); // is_adult
        let start_year = record[5].parse::<i32>().unwrap_or(0); // start_year
        let end_year = record[6].parse::<i32>().unwrap_or(0); // end_year
        let runtime = record[7].parse::<i32>().unwrap_or(0); // runtime_minutes
        sqlx::query!(r#"
            INSERT OR IGNORE INTO titles
                (tconst, title_type, primary_title, original_title, is_adult, start_year, end_year, runtime_minutes, genres)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#,
        record[0], // tconst,
        record[1], // title_type, 
        record[2], // primary_title, 
        record[3], // original_title, 
        is_adult, // is_adult, 
        start_year, // start_year, 
        end_year, // end_year, 
        runtime,// 
        record[8], // genres
        )
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}
pub async fn ingest_aka(
    transaction: &mut Transaction<'_, Sqlite>,
    record: &Vec<String>,
) -> Result<(), sqlx::Error> {
    if record.len() >= 8 {
        sqlx::query(
            r#"INSERT OR IGNORE INTO title_akas 
                (title_id, ordering, title, region, language, types, attributes, is_original_title)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(&record[0])
        .bind(record[1].parse::<i32>().unwrap_or(0))
        .bind(&record[2])
        .bind(&record[3])
        .bind(&record[4])
        .bind(&record[5])
        .bind(&record[6])
        .bind(record[7].parse::<i32>().unwrap_or(0))
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}
