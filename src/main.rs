extern crate rpassword;
use std::io;
use rpassword::read_password;
use std::io::Write;
use std::io::Cursor;
use std::path::Path;
use quit;
use std::process;

use bsky_sdk::BskyAgent;
use atrium_api::types::string::Datetime;
use bsky_sdk::moderation::decision::DecisionContext;
use atrium_api::app::bsky::feed::get_timeline::ParametersData;
use bsky_sdk::agent::config::{Config, FileStore};
use atrium_api::app::bsky::feed::post::RecordData as PostRecordData;
use serde_json::from_value;

struct Post{
    created_at: String,
    embed: String,
    entities: String,
    facets: String,
    labels: String,
    langs: String,
    reply: String,
    tags: String,
    text: String,
}
impl Post {
    fn print_post(){
        println!("{}[2J", 27 as char);

    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}[2J", 27 as char);
    println!("  /$$$$$$  /$$       /$$$$$$        /$$$$$$  /$$   /$$ /$$     /$$");
    println!(" /$$__  $$| $$      |_  $$_/       /$$__  $$| $$  /$$/|  $$   /$$/");
    println!("| $$  |__/| $$        | $$        | $$  |__/| $$ /$$/  |  $$ /$$/ ");
    println!("| $$      | $$        | $$        |  $$$$$$ | $$$$$/    |  $$$$/  ");
    println!("| $$      | $$        | $$         |____  $$| $$  $$     |  $$/   ");
    println!("| $$    $$| $$        | $$         /$$  | $$| $$|  $$     | $$    ");
    println!("|  $$$$$$/| $$$$$$$$ /$$$$$$      |  $$$$$$/| $$ |  $$    | $$    ");
    println!(" |______/ |________/|______/       |______/ |__/  |__/    |__/   ");
    println!("");
    println!("");

    if Path::new("config.json").exists() {
        println!("config.json exists!");
        ask_to_login().await?;
    } else {
        println!("config.json does not exist.");
    }
    login().await?;
    Ok(())
}

async fn login() -> Result<(), Box<dyn std::error::Error>> {
    println!("Welcome!!! Please enter your AT Protocol Handle");

    loop {
        println!("Login:");

        let mut uname = String::new();
        std::io::stdin().read_line(&mut uname).expect("Failed to read");

        println!("");
        println!("what is your password?");

   //     std::io::stdout().flush().unwrap();
     //   let pwd = read_password().unwrap();
       // println!("{pwd}");

       let pwd = rpassword::prompt_password("Password: ").unwrap();
       println!("{pwd}");

        let agent = create_agent(uname.trim().to_string(), pwd.to_string()).await?;


        match start_session(agent).await {
            Ok(_) => {
                // Session started successfully
                return Ok(());
            }
            Err(e) => {
                println!("\nLogin failed. Please check your handle and password.");
                println!("{e}");
                // Optionally, you could log the detailed error for debugging:
                // eprintln!("Error details: {}", e);
                continue;
            }
        }
    }
}

async fn ask_to_login() -> Result<(), Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder()
    .config(Config::load(&FileStore::new("config.json")).await?)
    .build()
    .await?;

    match start_session(agent).await {
        Ok(_) => {
            // Session started successfully
            return Ok(());
        }
        Err(e) => {
            println!("\nLogin failed. Please enter new details.");
            println!("{e}");
            login().await?;
            Ok(())
        }
    }
}

async fn create_agent(uname: String, pwd: String) -> Result<BskyAgent, Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder().build().await?;
    let session = agent.login(&uname,&pwd).await?;
    println!("Logged in! DID = {}", session.did.to_string());
    return Ok(agent);
}

async fn start_session(agent: BskyAgent) -> Result<(), Box<dyn std::error::Error>> {
    agent.to_config()
         .await
         .save(&FileStore::new("config.json"))
         .await?;
    println!("Session saved to config.json");

    match menu(agent).await {
        Ok(_) => {
            // Session started successfully
            return Ok(());
        }
        Err(e) => {
            println!("there is an error here");
            return Err(e);
        }
    }

}

async fn menu(agent: BskyAgent) -> Result<(), Box<dyn std::error::Error>> {
    //    print!("{}[2J", 27 as char);

    println!("What would you like to do?");
    println!("1: Text Post");
    println!("2: Following Feed");
    println!("");

    let mut input = String::new();

    loop{
        std::io::stdin().read_line(&mut input).expect("Failed to read menu input");

        match input.trim().parse::<i32>(){
            Ok(1) => {
                println!("writing post");
                make_post(agent).await?;
                break;
            }
            Ok(2) => {
                println!("following feed");
                following_feed(agent).await?;
                break;
            }

            _ => {
                input.clear(); // Clear input buffer for next read
                println!("Invalid input");
                continue;
            }
        };
    }

    Ok(())
}

async fn make_post(agent: BskyAgent) -> Result<(), Box<dyn std::error::Error>> {
    let mut content = String::new();
    io::stdin().read_line(&mut content).expect("Failed to read post content");


    agent
        .create_record(atrium_api::app::bsky::feed::post::RecordData {
            created_at: Datetime::now(),
            embed: None,
            entities: None,
            facets: None,
            labels: None,
            langs: None,
            reply: None,
            tags: None,
            text: content,
        })
        .await?;
    Ok(())
}

async fn following_feed(agent: BskyAgent)-> Result<(), Box<dyn std::error::Error>>{

    // Fetch the first page of timeline (default cursor, default limit)
    let output = agent
        .api
        .app
        .bsky
        .feed
        .get_timeline(
            ParametersData {
                cursor: None,
                limit: None,
                algorithm: Some("reverse-chronological".to_string()),
            }
            .into(),
        )
        .await?;

    // Iterate over posts
    for feed_post in &output.feed {
        let author = &feed_post.post.author;

        // Try deserializing the record field into a known post structure
        let record_value = serde_json::to_value::<atrium_api::types::Unknown>(feed_post.post.record.clone())?;
        let maybe_post: Result<PostRecordData, _> =
            from_value(record_value);

        match maybe_post {
            Ok(post_record) => {
                println!(
                    "{} (@{}): {}",
                    author.display_name.clone().unwrap_or_default(),
                    author.handle.to_string(),
                    post_record.text
                );
            }
            Err(e) => {
                eprintln!(
                    "Failed to parse post record from {} (@{}): {}",
                    author.display_name.clone().unwrap_or_default(),
                    author.handle.to_string(),
                    e
                );
            }
        }
    }
    process::exit(0);
    Ok(())
}
