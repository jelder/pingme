use anyhow::Result;
use clap::Parser;
use plist::Value;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
pub struct Cli {
    #[clap(default_value = "Hey")]
    message: String,
}

static PLIST_PATH: &str = "Library/Preferences/MobileMeAccounts.plist";

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut home_dir: PathBuf = home::home_dir().expect("Home directory not found");
    home_dir.push(PLIST_PATH);

    // let mut file = File::open(&home_dir)?;
    // let mut contents = String::new();
    // file.read_to_string(&mut contents)?;

    // let accounts = Value::from_file(home_dir).expect("failed to read book.plist");
    if let Some(account) = Value::from_file(&home_dir)
        .unwrap_or_else(|_| panic!("Failed to read {:?}", home_dir))
        .as_dictionary()
        .and_then(|dict| dict.get("Accounts"))
        .and_then(|accounts| accounts.as_array())
        .and_then(|accounts| {
            accounts.iter().find(|a| {
                a.as_dictionary()
                    .and_then(|a| a.get("LoggedIn"))
                    .and_then(|logged_in| logged_in.as_boolean())
                    .unwrap()
            })
        })
        .and_then(|account| account.as_dictionary())
        .and_then(|account| account.get("AccountID"))
        .and_then(|account_id| account_id.as_string())
    {
        send(account, &cli.message)?;
    } else {
        eprintln!("Couldn't find a logged-in iCloud account in {PLIST_PATH}");
        say(&cli.message)?;
    }

    Ok(())
}

fn send(account: &str, message: &str) -> Result<ExitStatus> {
    let script = format!(
        r#"
            tell application "Messages"
                set targetService to 1st account whose service type = iMessage
                set targetBuddy to participant "{account}" of targetService
                send "{message}" to targetBuddy
            end tell
        "#
    );

    let mut child = Command::new("osascript")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(ref mut stdin) = child.stdin {
        stdin.write_all(script.as_bytes())?;
    }

    Ok(child.wait()?)
}

fn say(message: &str) -> Result<ExitStatus> {
    let mut child = Command::new("say")
        .arg(message)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(child.wait()?)
}
