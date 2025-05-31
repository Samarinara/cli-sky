use std::fs::File;
use std::io::Write;
use std::process;
use std::fs;
use std::io::{self, BufRead};

use bsky_cli::com;
use keyring::error::Error;
use bsky_sdk::BskyAgent;
use atrium_api::types::string::Datetime;
use atrium_api::app::bsky::feed::get_timeline::ParametersData;
use bsky_sdk::agent::config::{Config, FileStore};
use atrium_api::app::bsky::feed::post::RecordData as PostRecordData;
use serde_json::from_value;
use serde::{Deserialize, Serialize};
use bsky_cli::lexicon::record::KnownRecord;
use bsky_cli::lexicon::wrapper::AtpServiceClientWrapper;
use atrium_api::com::atproto::repo::create_record::InputData;

async fn print_logo() {
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
}

#[derive(Serialize, Deserialize)]
struct BlogPost {
    title: String,
    text: String,
    tags: Option<Vec<String>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    print_logo().await;

    ask_to_login().await?;
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
    let service = "cli_sky";
    let username = "user";
    let entry = keyring::Entry::new(service, username)?;

    let mut pwd = String::new();


    match entry.get_password() {
        Ok(secret) => {pwd = secret},
        Err(Error::NoEntry) => {
            login().await?;
        }
        Err(e) => {
            eprintln!("Failed to get password: {}", e);
        }
    }

    println!("Got secret: {pwd}");

    let mut file = File::create("config.json")?;
    file.write_all(pwd.as_bytes())?;

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
    save_session(&agent).await?;


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

async fn save_session(agent: &BskyAgent) -> Result<(), Box<dyn std::error::Error>> {
    agent.to_config()
    .await
    .save(&FileStore::new("config.json"))
    .await?;
    println!("Session saved to config.json");

    //deserialize the json
    let config = fs::read_to_string("config.json")?;

    //create an entry
    let service = "cli_sky";
    let username = "user";
    let entry = keyring::Entry::new(service, username)?;
    entry.set_password(&config)?;

    //delete the config.json file
    fs::remove_file("config.json")?;

    Ok(())
}

fn menu(agent: BskyAgent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Box<dyn std::error::Error>>> + Send>> {
    Box::pin(async move {
        print!("{}[2J", 27 as char);
        print_logo().await;
        println!("");
        println!("What would you like to do?");
        println!("0: Exit");
        println!("1: Text Post");
        println!("2: Following Feed");
        println!("3: Blog Post");
        println!("");

        let mut input = String::new();

        loop {
            std::io::stdin().read_line(&mut input).expect("Failed to read menu input");

            match input.trim().parse::<i32>() {
                Ok(0) => {
                    println!("Exiting...");
                    process::exit(0);
                }
                Ok(1) => {
                    println!("writing post");
                    make_post(agent.clone()).await?;  // Assuming agent is cloneable
                    break;
                }
                Ok(2) => {
                    println!("following feed");
                    following_feed(agent.clone()).await?;
                    break;
                }
                Ok(3) => {
                    println!("blog post");
                    write_blog(agent.clone()).await?;
                    break;
                }
                _ => {
                    input.clear();
                    println!("Invalid input");
                    continue;
                }
            };
        }

        Ok(())
    })
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

    println!("Post sent to the Atmosphere!!!");
    menu(agent).await?;
    Ok(())
}

async fn following_feed(agent: BskyAgent)-> Result<(), Box<dyn std::error::Error>>{

    // Fetch the first page of timeline (default cursor, default limit)
    let output = agent.clone()
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
                println!("{}[2J", 27 as char);
                println!(
                    "{} \n(@{})\n\n {}",
                    author.display_name.clone().unwrap_or_default(),
                    author.handle.to_string(),
                    post_record.text
                );
                println!("\nType 'exit' to quit.\n");


                let mut input = String::new();
                std::io::stdin().read_line( &mut input).expect("Failed to read");
                match input.trim() {
                    "exit" => {process::exit(0)}
                    "menu" => {menu(agent.clone()).await?},
                    _ => {}
                }
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
}

async fn write_blog(agent: BskyAgent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter blog title:");
    let mut title = String::new();
    io::stdin().read_line(&mut title)?;
    let title = title.trim().to_string();
    
    println!("Paste blog content in markdown (type END in all caps on a new line to finish):");
    let stdin = io::stdin();
    let mut content = String::new();
    
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim() == "END" { break; }
        content.push_str(&line);
        content.push('\n');
    }
    let content = content.trim().to_string();
    
    println!("Enter tags (comma-separated, or press Enter for none):");
    let mut tags_input = String::new();
    io::stdin().read_line(&mut tags_input)?;
    let tags: Option<Vec<String>> = if tags_input.trim().is_empty() {
        None
    } else {
        Some(tags_input.trim().split(',').map(|s| s.trim().to_string()).collect())
    };

    let blog_record = BlogPost {
        title: title.clone(),
        text: content.clone(),
        tags: tags.clone(),
    };

    let record_json = serde_json::json!({
        "$type": "com.macroblog.blog.post",
        "title": title,
        "text": content,
        "tags": tags,
    });

    let session_info = agent.api.com.atproto.server.get_session().await?;
    let did = session_info.did.clone(); // or however you get the current user's DID

    let record_unknown: atrium_api::types::Unknown = serde_json::from_value(record_json)?;
    let input = InputData {
        collection: "com.macroblog.blog.post".parse()?, // Your lexicon's NSID
        repo: did.into(),
        rkey: None, // Let the server pick a key
        record: record_unknown,
        swap_commit: None, 
        validate: None,
    };
    agent.api.com.atproto.repo.create_record(input.into()).await?;

    println!("Blog post created successfully!");
    menu(agent).await?;
    Ok(())
} 


