use anyhow::Result;
use serde::Serialize;
use sqlx::{prelude::FromRow, sqlite::SqliteRow, Pool, Row, Sqlite, SqlitePool, Transaction};

#[derive(Debug, Serialize)]
pub struct Name {
    pub nconst: String,
    pub primary_name: String,
    pub birth_year: Option<i32>,
    pub death_year: Option<i32>,
    pub primary_profession: Vec<String>,
    pub known_for_titles: Vec<String>,
}

impl<'r> FromRow<'r, SqliteRow> for Name {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let profession_str = row.try_get("primary_profession").unwrap_or("".to_owned());
        let primary_profession = profession_str.split(',').map(|s| s.to_owned()).collect();
        let titles_str = row.try_get("known_for_titles").unwrap_or("".to_owned());
        let known_for_titles = titles_str.split(',').map(|s| s.to_owned()).collect();

        Ok(Self {
            nconst: row.try_get("tconst").unwrap_or("".into()),
            primary_name: row.try_get("primary_name").unwrap_or("".into()),
            birth_year: row.try_get("birth_year").unwrap_or(None),
            death_year: row.try_get("death_year").unwrap_or(None),
            primary_profession,
            known_for_titles,
        })
    }
}

pub async fn init_table(pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
    // Create all necessary tables
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS names (
                nconst TEXT PRIMARY KEY,
                primary_name TEXT,
                birth_year INTEGER,
                death_year INTEGER,
                primary_profession TEXT,
                known_for_titles TEXT
            )",
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn primary_name(db: &SqlitePool, id: String) -> Result<String> {
    // TODO check if there is anything else in query and error
    if id.is_empty() {
        return Err(super::DBError::new("empty id".into()).into());
    }
    let name = sqlx::query_as::<_, Name>("SELECT * FROM names WHERE nconst = ?")
        .bind(id)
        .fetch_one(db)
        .await?;
    Ok(name.primary_name)
}

// pub struct NameQuery<'a>(QueryBuilder<'a, Sqlite>);
// impl<'a> NameQuery<'a> {
//     pub fn new() -> Self {
//         NameQuery(QueryBuilder::new("SELECT * FROM names"))
//     }

//     pub fn id(mut self, id: &'a String) -> Self {
//         if !id.is_empty() {
//             self.where_and();
//             self.0.push(" nconst = ");
//             self.0.push_bind(id);
//         }
//         self
//     }

//     fn where_and(&mut self) {
//         if !self.0.sql().contains("WHERE") {
//             self.0.push(" WHERE");
//         } else {
//             self.0.push(" AND");
//         }
//     }

//     fn known_for_titles(mut self, tconst: &'a String) -> Self {
//         if !tconst.is_empty() {
//             self.where_and();
//             self.0.push(" known_for_titles CONTAINS ");
//             self.0.push_bind(tconst);
//         }
//         self
//     }

//     pub async fn fetch_one(mut self, db: &SqlitePool) -> Result<Name> {
//         Ok(self.0.build_query_as::<Name>().fetch_one(db).await?)
//     }
// }

pub async fn ingest(
    transaction: &mut Transaction<'_, Sqlite>,
    record: &Vec<String>,
) -> Result<(), sqlx::Error> {
    if record.len() >= 6 {
        sqlx::query(
            "INSERT OR IGNORE INTO names 
            (nconst, primary_name, birth_year, death_year, primary_profession, known_for_titles)
            VALUES (?, ?, ?, ?, ?, ?)",
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
        .bind(&record[4])
        .bind(&record[5])
        .execute(&mut **transaction)
        .await?;
    }
    Ok(())
}
