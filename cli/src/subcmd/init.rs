use std::{
    collections::HashMap,
    fs::{create_dir_all, OpenOptions},
    io::Write,
};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

use dialoguer::{Confirmation, Input};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use toml::to_string;

use crate::{schema::UserInfo, Error, Result, PATHS};

pub fn init() -> Result<()> {
    println!("Welcome to frauth!");

    if !Confirmation::new()
        .with_text("Ready to get started?")
        .interact()?
    {
        return Err(Error::from("Halting init"));
    }

    let already_exists = PATHS.user_info.exists();

    if already_exists {
        println!("\nIt looks like you've already initialized frauth.");
        println!("Do you want to re-initialize? THIS WILL ERASE YOUR EXISTING KEYS AND DATA!");

        if !Confirmation::new()
            .default(false)
            .with_text("Continue?")
            .interact()?
        {
            return Err(Error::from("Halting init"));
        }
    }

    println!("\nCreating directories...");

    create_dir_all(&PATHS.base_data)?;
    create_dir_all(&PATHS.base_cache)?;

    // Create files early to prevent late errors
    let mut opt = OpenOptions::new();
    opt.write(true);
    opt.truncate(true);
    opt.create(true);

    #[cfg(unix)]
    opt.mode(0o600);

    // TODO: Figure out how file permissions work on Windows, or link to an issue
    #[cfg(not(unix))]
    eprintln!(
        "Warning! You should set the permissions for {} to only be readable by this user!"
        PATHS.user_info.display()
    );

    let mut file = opt.open(&PATHS.user_info)?;

    println!("Done.");

    println!("\nOkay! We'll get started by collecting some required info.");

    let name = Input::<String>::new()
        .with_prompt("What name do you want to go by?")
        .interact()?;

    println!("\nOkay, that's everything that's required. Now let's collect some optional items.");

    println!("\nWe'll now collect any identities you'd like to associate with yourself. You can add as many as you like.");
    println!("These identities will be publicly visible to anyone.");

    println!("\nIdentities have a 'name', like 'twitter', 'email', 'mobile', etc.");
    println!("and an 'id', like 'my_twitter_id', 'me@example.com', or '+4912345678901'.");

    let mut identities: HashMap<String, String> = HashMap::new();

    loop {
        if !Confirmation::new()
            .with_text("\nAdd/Update an identity?")
            .interact()?
        {
            break;
        }

        let id_name = Input::<String>::new()
            .with_prompt("\nIdentity name")
            .interact()?;
        let id_val = Input::<String>::new()
            .with_prompt(&format!("{} id", id_name))
            .interact()?;
        identities.insert(id_name, id_val);

        println!("\nCurrent identities: {:#?}", identities);
    }

    println!("\nWould you like to add a public status message? You can change or add this later as well.");

    let status = if Confirmation::new()
        .with_text("\nAdd a status?")
        .interact()?
    {
        // TODO: Use an editor instead, limit the character length
        Some(Input::<String>::new().with_prompt("\nStatus").interact()?)
    } else {
        None
    };

    let keypair = Keypair::generate(&mut OsRng);

    let user_info = UserInfo {
        name,
        identities,
        status,
        keypair,
    };

    let contents = to_string(&user_info)?;

    file.write_all(USER_INFO_HEADER.as_bytes())?;
    file.write_all(contents.as_bytes())?;

    println!("\nfrauth has been initialized!");

    // TODO: add next steps, like using `frauth me`

    Ok(())
}

const USER_INFO_HEADER: &str = r#"
# User information, generated by a really early version of frauth.
#
# THIS FILE CONTAINS SECRET DATA! You should never post or share it anywhere!
#
# Instead, use command (TODO) to generate the file to publish publicly.
"#;
