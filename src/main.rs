use byte_unit::Byte;
use exitcode;
use glob::glob;
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

const FIRST_ATTACHMENTS_METADATA_FILENAME: &str = "attachments_000001.json";
const ATTACHMENTS_DIRECTORY_NAME: &str = "attachments";

fn read_attachments_file(path: PathBuf) -> Result<Vec<Attachment>, std::io::Error> {
    let attachments_json = std::fs::read_to_string(&path)?;
    let attachments: Vec<Attachment> = serde_json::from_str(&attachments_json).unwrap();

    Ok(attachments)
}

fn read_attachments_files(working_directory: &PathBuf) -> Result<Vec<Attachment>, std::io::Error> {
    let mut attachments: Vec<Attachment> = Vec::new();

    for entry in glob(
        &working_directory
            .join("attachments_*.json")
            .to_str()
            .unwrap(),
    )
    .unwrap()
    {
        match entry {
            Ok(path) => {
                let mut file_attachments = read_attachments_file(path)?;
                attachments.append(&mut file_attachments);
            }
            Err(e) => panic!("Unexpected GlobError: {:?}", e),
        }
    }

    Ok(attachments)
}

fn get_working_directory(working_directory_path: Option<String>) -> PathBuf {
    if working_directory_path.is_none() {
        return PathBuf::from(".");
    } else {
        return PathBuf::from(working_directory_path.unwrap());
    }
}

fn process_attachments(
    provided_working_directory: Option<String>,
) -> Result<Vec<String>, std::io::Error> {
    let working_directory = get_working_directory(provided_working_directory);

    let first_attachments_metadata_path =
        working_directory.join(FIRST_ATTACHMENTS_METADATA_FILENAME);
    let attachments_directory_path = working_directory.join(ATTACHMENTS_DIRECTORY_NAME);

    if !first_attachments_metadata_path.exists() || !attachments_directory_path.exists() {
        let error_mesage = format!("Could not find `{}` file and/or `{}/` directory. This suggests that either (a) your archive contains no attachments or (b) you're not in a directory created when you extract a GitHub archive.", first_attachments_metadata_path.display(), attachments_directory_path.display());
        return Err(Error::new(ErrorKind::Other, error_mesage));
    }

    eprintln!("📖 Reading attachments metadata files to find attachments...");

    let attachments: Vec<Attachment> = match read_attachments_files(&working_directory) {
        Ok(attachments) => attachments,
        Err(e) => {
            let error_mesage = format!("Could not read attachments metadata files: {}", e);
            return Err(Error::new(ErrorKind::Other, error_mesage));
        }
    };

    let attachments_count = attachments.len();
    eprintln!("🔎 Found {} attachment(s)", attachments_count);

    let mut attachments_by_size: Vec<(&Attachment, u128)> = attachments
        .iter()
        .enumerate()
        .map(|(index, attachment)| {
            eprintln!(
                "📜 Processing attachment {}/{}",
                index + 1,
                attachments_count
            );

            let relative_path = Path::new(&working_directory).join(attachment.asset_url.replace("tarball://root/", ""));

            if !relative_path.exists() {
                panic!("Could not find listed attachment file `{}`. Please make sure you're running this tool in the directory created when you extract a GitHub archive.", relative_path.display());
            }

            let size = fs::metadata(&relative_path).unwrap().len() as u128;
            return (attachment, size);
        })
        .collect::<Vec<(&Attachment, u128)>>();

    eprintln!("🪣  Sorting attachments by size...");

    // Sort the attachments by size, largest first. This is done in memory. I haven't figured out how
    // to do an immutable sort yet.
    attachments_by_size.sort_unstable_by_key(|attachment_and_size| attachment_and_size.1);
    attachments_by_size.reverse();

    // Accumulate the messages to print. We do this instead of directly looping and printing messages as
    // we go becuase it allows us to print warning messages first, before the actual results.
    let messages: Vec<String> = attachments_by_size.iter().fold(Vec::new(), |mut messages, (attachment, size)| {
        let byte = Byte::from_bytes(*size);
        let adjusted_byte = byte.get_appropriate_unit(false);
        let size_as_string = adjusted_byte.format(1);

        if attachment.pull_request.is_some()
        {
            messages.push(format!("{} ({}) - {}", attachment.asset_name, &attachment.pull_request.clone().unwrap(), size_as_string));
        } else if attachment.issue.is_some() {
            messages.push(format!("{} ({}) - {}", attachment.asset_name, &attachment.issue.clone().unwrap(), size_as_string));
        } else if attachment.issue_comment.is_some() {
            messages.push(format!("{} ({}) - {}", attachment.asset_name, &attachment.issue_comment.clone().unwrap(), size_as_string));
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
    fn it_identifies_attachments_in_single_file() {
        let result = super::process_attachments(Some("fixtures/single-file".to_string()));

        match result {
            Ok(val) => {
                assert_eq!(val, vec!["todd-trapani-QldMpmrmWuc-unsplash.jpg (https://github.com/caffeinesoftware/rewardnights/pull/337) - 144.1 KB"])
            }
            Err(e) => {
                panic!("process_attachments returned an error: {}", e)
            }
        }
    }

    #[test]
    fn it_identifies_attachments_across_multiple_files() {
        let result = super::process_attachments(Some("fixtures/multiple-files".to_string()));

        match result {
            Ok(val) => {
                assert_eq!(val, vec![
                    "todd-trapani-QldMpmrmWuc-unsplash-2.jpg (https://github.com/caffeinesoftware/rewardnights/pull/337) - 144.1 KB",
                    "todd-trapani-QldMpmrmWuc-unsplash.jpg (https://github.com/caffeinesoftware/rewardnights/pull/337) - 144.1 KB"
                ])
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
