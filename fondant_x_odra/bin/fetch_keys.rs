use reqwest::blocking::Client; // Use blocking client for simplicity
use serde_json::Value;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = "http://localhost:3001";
    let start_id = 1;
    let end_id = 5;

    let client = Client::new();
    let key_dir = Path::new(".keys");
    create_dir_all(key_dir)?;

    for id in start_id..=end_id {
        let url = format!("{}/users/{}/private_key", base_url, id);
        let filename = key_dir.join(format!("secret_key_{}.pem", id));

        // Fetch the JSON data
        let response = client.get(&url).send()?;
        let json_response: Value = response.json()?;

        // Extract and save the private key
        if let Some(message) = json_response.get("message").and_then(|v| v.as_str()) {
            let mut file = File::create(&filename)?; // Borrow filename with &
            file.write_all(message.as_bytes())?;
            println!("Saved key {} to {}", id, filename.display());
        } else {
            eprintln!("Error: Private key not found in response for {}", url);
        }
    }

    Ok(())
}
