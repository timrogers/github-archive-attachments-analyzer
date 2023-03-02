use exitcode;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;

#[derive(Deserialize, Serialize, Debug)]
struct Attachment {
    r#type: String,
    url: String,
    pull_request: Option<String>,
    issue: Option<String>,
    issue_comment: Option<String>,
    user: String,
    asset_name: String,
    asset_content_type: String,
    asset_url: String,
    created_at: String,
}

const INPUT_PATH: &str = "attachments_000001.json";
const ATTACHMENTS_PATH: &str = "attachments";

fn process_attachments() -> Result<Vec<String>, std::io::Error> {
    if !Path::new(&INPUT_PATH).exists() || !Path::new(&ATTACHMENTS_PATH).exists() {
        let error_mesage = format!("Could not find `{}` file and/or `{}/` directory. Please make sure you're running this tool in the directory created when you extract a GitHub archive.", INPUT_PATH, ATTACHMENTS_PATH);
        return Err(Error::new(ErrorKind::Other, error_mesage));
    }

    println!("📖 Reading {} to find attachments...", INPUT_PATH);

    // Parse the attachments JSON file into a vector of Attachment structs
    let attachments_json = std::fs::read_to_string(&INPUT_PATH)?;
    let attachments: Vec<Attachment> = serde_json::from_str(&attachments_json).unwrap();

    let attachments_count = attachments.len();
    println!("🔎 Found {} attachment(s)", attachments_count);

    let mut attachments_by_size: Vec<(&Attachment, u64)> = attachments
        .iter()
        .enumerate()
        .map(|(index, attachment)| {
            println!(
                "📜 Processing attachment {}/{}",
                index + 1,
                attachments_count
            );

            let relative_path = attachment.asset_url.replace("tarball://root/", "");

            if !Path::new(&relative_path).exists() {
                panic!("Could not find listed attachment file `{}`. Please make sure you're running this tool in the directory created when you extract a GitHub archive.", relative_path);
            }

            let size = fs::metadata(&relative_path).unwrap().len();
            return (attachment, size);
        })
        .collect::<Vec<(&Attachment, u64)>>();

    println!("🪣  Sorting attachments by size...");

    // Sort the attachments by size, largest first. This is done in memory. I haven't figured out how
    // to do an immutable sort yet.
    attachments_by_size.sort_unstable_by_key(|attachment_and_size| attachment_and_size.1);
    attachments_by_size.reverse();

    // Accumulate the messages to print. We do this instead of directly looping and printing messages as
    // we go becuase it allows us to print warning messages first, before the actual results.
    let messages: Vec<String> = attachments_by_size.iter().fold(Vec::new(), |mut messages, (attachment, size)| {
        if attachment.pull_request.is_some()
        {
            messages.push(format!("{} ({}) - {} bytes", attachment.asset_name, &attachment.pull_request.clone().unwrap(), size));
        } else if attachment.issue.is_some() {
            messages.push(format!("{} ({}) - {} bytes", attachment.asset_name, &attachment.issue.clone().unwrap(), size));
        } else if attachment.issue_comment.is_some() {
            messages.push(format!("{} ({}) - {} bytes", attachment.asset_name, &attachment.issue_comment.clone().unwrap(), size));
        } else {
            println!("⚠️ Could not find issue, pull request or issue comment for attachment {}. Skipping...", attachment.asset_name);
        }

        return messages;
    });

    Ok(messages)
}

fn main() -> Result<(), std::io::Error> {
    let result = process_attachments();

    match result {
        Ok(messages) => {
            for message in messages.iter() {
                println!("{}", message)
            }

            std::process::exit(exitcode::OK);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(exitcode::DATAERR);
        }
    }
}
