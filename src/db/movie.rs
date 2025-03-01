use futures::future;

use anyhow::Result;
use serde::Serialize;
use sqlx::SqlitePool;

use super::{
    crew::{self, Crew},
    names, principals, titles,
};

#[derive(Serialize)]
pub struct Movie {
    title: String,
    year: i64,
    crew: Crew,
    principals: Vec<(String, String)>,
}

pub async fn get(db: &SqlitePool, tconst: String) -> Result<Movie> {
    let title = titles::TitleQuery::new().id(&tconst).fetch_one(&db).await?;

    let crew = crew::CrewQuery::new().id(&tconst).fetch_one(&db).await?;
    let principals = principals::PrincipalsQuery::new()
        .movie(&tconst)
        .fetch(&db)
        .await?;
    let director = names::NameQuery::new()
        .id(&crew.directors[0])
        .fetch_one(&db)
        .await?;
    let principals = future::join_all(
        principals
            .into_iter()
            .map(|p| async { (names::primary_name(db, p.nconst).await.unwrap(), p.job) })
            .collect::<Vec<_>>(),
    )
    .await;
    println!("directed by: {:?} {:?}", director.primary_name, principals);

    Ok(Movie {
        title: title.primary_title,
        year: title.start_year,
        principals,
        crew,
    })

    //         r#"SELECT t.tconst,
    //         n.primary_name,
    //         p.characters,
    //         p.category,
    //         p.job
    //         FROM titles AS t
    //         JOIN principals AS p ON p.tconst = t.tconst
    //         JOIN names AS n ON p.nconst = n.nconst
    //         WHERE t.tconst=(SELECT tconst FROM titles WHERE primary_title LIKE ?);
    // "#
}
