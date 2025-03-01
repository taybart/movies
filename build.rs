use flate2::read::GzDecoder;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::Path,
};

const IMDB_FILES: &[&str] = &[
    "name.basics.tsv",
    "title.akas.tsv",
    "title.basics.tsv",
    "title.crew.tsv",
    "title.episode.tsv",
    "title.principals.tsv",
];

fn download_file(
    client: &Client,
    url: &str,
    output_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a request to get the content length
    let response = client.get(url).send()?;
    let total_size = response.content_length().unwrap_or(0);

    // Create progress bar
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    // Stream the download with progress updates
    let mut response = client.get(url).send()?;
    let mut output_file = File::create(output_path)?;
    let mut buffer = Vec::new();
    response.read_to_end(&mut buffer)?;

    pb.finish_with_message("Download completed");

    // Decompress and write to file
    println!("Decompressing {}...", output_path.display());
    let mut decoder = GzDecoder::new(&buffer[..]);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)?;
    output_file.write_all(&decompressed)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell Cargo to rerun if any of the data files change
    for file in IMDB_FILES {
        println!("cargo:rerun-if-changed=data/{}", file);
    }

    // Create data directory if it doesn't exist
    fs::create_dir_all("data")?;

    // Create HTTP client
    let client = Client::new();

    for file in IMDB_FILES {
        let file_path = Path::new("data").join(file);

        if !file_path.exists() {
            println!("Processing {}...", file);

            // Construct the download URL
            let url = format!("https://datasets.imdbws.com/{}.gz", file);

            // Download and decompress the file
            match download_file(&client, &url, &file_path) {
                Ok(_) => println!("Successfully processed {}", file),
                Err(e) => eprintln!("Error processing {}: {}", file, e),
            }
        }
    }

    Ok(())
}
