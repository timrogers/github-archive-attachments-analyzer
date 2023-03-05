use exitcode;
use serde_derive::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::path::PathBuf;

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

fn process_attachments(
    working_directory_path: Option<String>,
) -> Result<Vec<String>, std::io::Error> {
    let input_path: PathBuf;
    let attachments_path: PathBuf;

    if working_directory_path.is_none() {
        input_path = PathBuf::from(INPUT_PATH);
        attachments_path = PathBuf::from(ATTACHMENTS_PATH);
    } else {
        let path = working_directory_path.as_ref().unwrap();
        input_path = Path::new(&path).join(INPUT_PATH);
        attachments_path = Path::new(&path).join(ATTACHMENTS_PATH);
    }

    if !input_path.exists() || !attachments_path.exists() {
        let error_mesage = format!("Could not find `{}` file and/or `{}/` directory. This suggests that either (a) your archive contains no attachments or (b) you're not in a directory created when you extract a GitHub archive.", input_path.display(), attachments_path.display());
        return Err(Error::new(ErrorKind::Other, error_mesage));
    }

    eprintln!("📖 Reading {} to find attachments...", input_path.display());

    // Parse the attachments JSON file into a vector of Attachment structs
    let attachments_json = std::fs::read_to_string(&input_path)?;
    let attachments: Vec<Attachment> = serde_json::from_str(&attachments_json).unwrap();

    let attachments_count = attachments.len();
    eprintln!("🔎 Found {} attachment(s)", attachments_count);

    let mut attachments_by_size: Vec<(&Attachment, u64)> = attachments
        .iter()
        .enumerate()
        .map(|(index, attachment)| {
            eprintln!(
                "📜 Processing attachment {}/{}",
                index + 1,
                attachments_count
            );

            let relative_path: PathBuf;

            if working_directory_path.is_some() {
                let path = working_directory_path.as_ref().unwrap();
                relative_path = Path::new(&path).join(attachment.asset_url.replace("tarball://root/", ""));
            } else {
                relative_path = PathBuf::from(attachment.asset_url.replace("tarball://root/", ""));
            }

            if !relative_path.exists() {
                panic!("Could not find listed attachment file `{}`. Please make sure you're running this tool in the directory created when you extract a GitHub archive.", relative_path.display());
            }

            let size = fs::metadata(&relative_path).unwrap().len();
            return (attachment, size);
        })
        .collect::<Vec<(&Attachment, u64)>>();

    eprintln!("🪣  Sorting attachments by size...");

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
            eprintln!("⚠️ Could not find issue, pull request or issue comment for attachment {}. Skipping...", attachment.asset_name);
        }

        return messages;
    });

    Ok(messages)
}

fn main() -> Result<(), std::io::Error> {
    let result = process_attachments(None);

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_identifies_attachments() {
        let result = super::process_attachments(Some("fixtures".to_string()));

        match result {
            Ok(val) => {
                assert_eq!(val, vec!["todd-trapani-QldMpmrmWuc-unsplash.jpg (https://github.com/caffeinesoftware/rewardnights/pull/337) - 144106 bytes"])
            }
            Err(e) => {
                panic!("process_attachments returned an error: {}", e)
            }
        }
    }

    #[test]
    fn it_errors_if_expected_files_are_not_present() {
        let result = super::process_attachments(Some("src".to_string()));

        match result {
            Ok(_val) => {
                panic!("process_attachments returned a value, but was expected to error");
            }
            Err(e) => {
                assert_eq!(e.to_string(), "Could not find `src/attachments_000001.json` file and/or `src/attachments/` directory. This suggests that either (a) your archive contains no attachments or (b) you're not in a directory created when you extract a GitHub archive.".to_string());
            }
        }
    }
}
