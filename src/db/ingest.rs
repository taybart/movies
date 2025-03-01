use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite, Transaction};
use std::fs::File;
use std::io::{BufRead, BufReader};

use super::{crew, episodes, names, principals, titles};

pub struct IngestClient {
    pool: Pool<Sqlite>,
}

impl IngestClient {
    pub async fn new(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::query("PRAGMA journal_mode=WAL;")
            .execute(&pool)
            .await?;

        Ok(IngestClient { pool })
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Process all files
        let files = vec![
            ("data/name.basics.tsv", "names"),
            ("data/title.akas.tsv", "title_akas"),
            ("data/title.basics.tsv", "titles"),
            ("data/title.episode.tsv", "episodes"),
            ("data/title.principals.tsv", "principals"),
            ("data/title.crew.tsv", "crew"),
        ];

        for (filename, table_name) in files {
            if let Err(e) = self.process_file(filename, table_name).await {
                eprintln!("Error processing {}: {}", filename, e);
            }
        }
        Ok(())
    }

    async fn ingest_batch(
        transaction: &mut Transaction<'_, Sqlite>,
        table_name: &str,
        batch: &Vec<Vec<String>>,
    ) -> Result<(), sqlx::Error> {
        match table_name {
            "title_akas" => {
                for record in batch {
                    titles::ingest_aka(transaction, record).await?;
                }
            }
            "titles" => {
                for record in batch {
                    titles::ingest(transaction, record).await?;
                }
            }
            "names" => {
                for record in batch {
                    names::ingest(transaction, record).await?;
                }
            }
            "episodes" => {
                for record in batch {
                    episodes::ingest(transaction, record).await?;
                }
            }
            "principals" => {
                for record in batch {
                    principals::ingest(transaction, record).await?;
                }
            }

            "crew" => {
                for record in batch {
                    crew::ingest(transaction, record).await?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn process_file(
        &self,
        filename: &str,
        table_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Processing {}", filename);
        let file = File::open(filename)?;
        let reader = BufReader::new(file);
        let mut batch: Vec<Vec<String>> = Vec::with_capacity(super::INGEST_BATCH_SIZE);
        let mut total_records = 0;

        for line in reader.lines().skip(1) {
            if let Ok(line) = line {
                let fields: Vec<String> = line.split('\t').map(|s| s.to_string()).collect();

                batch.push(fields);

                if batch.len() >= super::INGEST_BATCH_SIZE {
                    let mut transaction = self.pool.begin().await?;
                    Self::ingest_batch(&mut transaction, table_name, &batch).await?;
                    transaction.commit().await?;

                    total_records += batch.len();
                    println!("{}: Processed {} records", table_name, total_records);
                    batch.clear();
                }
            }
        }

        if !batch.is_empty() {
            let mut transaction = self.pool.begin().await?;
            Self::ingest_batch(&mut transaction, table_name, &batch).await?;
            transaction.commit().await?;

            total_records += batch.len();
            println!("{}: Processed {} records", table_name, total_records);
        }

        Ok(())
    }
}
