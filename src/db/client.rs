use anyhow::Result;
use sqlx::SqlitePool;

use super::{crew, episodes, names, principals, titles};

pub async fn init_tables(db: &SqlitePool) -> Result<(), sqlx::Error> {
    names::init_table(db).await?;
    titles::init_table(db).await?;
    episodes::init_table(db).await?;
    principals::init_table(db).await?;
    crew::init_table(db).await?;
    Ok(())
}

#[tokio::test]
async fn test_title() -> Result<()> {
    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:movies.db")
        .await?;

    let title = sqlx::query_as::<_, super::titles::Title>(
        r#"select * from titles where tconst='tt0317705'"#,
    )
    .fetch_one(&pool)
    .await?;
    println!("{title:?}");

    Ok(())
}

#[tokio::test]
async fn test_join() -> Result<()> {
    use sqlx::{sqlite::SqlitePoolOptions, FromRow};
    #[derive(Debug, FromRow)]
    #[allow(dead_code)]
    struct Actor {
        tconst: Option<String>,
        primary_name: Option<String>,
        characters: Option<String>,
        category: Option<String>,
        job: Option<String>,
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite:movies.db")
        .await?;

    let title = sqlx::query_as::<_, super::titles::Title>(
        r#"SELECT * FROM titles AS t WHERE  primary_title LIKE ?"#,
    )
    .bind("The Godfather")
    .fetch_one(&pool)
    .await?;
    let actors = sqlx::query_as!(
        Actor,
        r#"
            SELECT t.tconst, 
            n.primary_name,
            p.characters,
            p.category,
            p.job
            FROM titles AS t 
            JOIN principals AS p ON p.tconst = t.tconst
            JOIN names AS n ON p.nconst = n.nconst
            WHERE t.tconst=(SELECT tconst FROM titles WHERE primary_title LIKE ?);
    "#,
        "The Godfather",
    )
    .fetch_all(&pool)
    .await?;
    println!("{:?}", title);
    for actor in actors {
        println!(
            "{}, played {}\n {} {}",
            actor.primary_name.unwrap(),
            actor.characters.unwrap(),
            actor.category.unwrap(),
            actor.job.unwrap()
        );
    }
    // println!("{}", movie[0].primary_title);

    // assert_eq!(part2(&elfs), 201524);
    Ok(())
}
