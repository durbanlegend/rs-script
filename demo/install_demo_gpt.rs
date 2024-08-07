/*[toml]
[dependencies]
log = "0.4.21"
reqwest = { version = "0.12.4", features = ["blocking", "json"] }
rfd = "0.14.1"
rs-script = { path = "/Users/donf/projects/rs-script" }

serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
*/
use reqwest::blocking::get;
use rfd::FileDialog;
use serde::Deserialize;
use std::fs::File;
use std::io::copy;

#[derive(Deserialize)]
struct GitHubFile {
    name: String,
    download_url: Option<String>,
    #[serde(rename = "type")]
    file_type: String,
}

fn get_github_files(repo: &str, path: &str) -> Result<Vec<GitHubFile>, Box<dyn std::error::Error>> {
    let url = format!("https://api.github.com/repos/{}/contents/{}", repo, path);
    let client = reqwest::blocking::Client::new();
    let response = client
        .get(&url)
        .header("User-Agent", "request")
        .send()?
        .json::<Vec<GitHubFile>>()?;

    Ok(response)
}

fn download_demo_files(repo: &str, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Display file chooser dialogue to select the destination directory
    let dest_dir = FileDialog::new()
        .set_directory(".")
        .pick_folder()
        .ok_or("No folder selected")?;
    println!(
        "Downloading starter kit files to {} ...",
        dest_dir.display()
    );

    let files = get_github_files(repo, path)?;

    for file in files {
        if file.file_type == "file" {
            if let Some(download_url) = file.download_url {
                let response = get(&download_url)?;
                let dest_path = dest_dir.join(&file.name);
                let mut dest_file = File::create(dest_path)?;
                println!("{}...", file.name);
                copy(&mut response.bytes()?.as_ref(), &mut dest_file)?;
            }
        }
    }

    Ok(())
}

/// This may seem bit of a catch-22 place to keep this program, but it's just a prototype.
fn main() {
    let repo = "durbanlegend/rs-script";
    let path = "demo";

    match download_demo_files(repo, path) {
        Ok(_) => println!("Demo files downloaded successfully."),
        Err(e) => eprintln!("Error downloading demo files: {}", e),
    }
}
