extern crate rpassword;
use std::io;
use rpassword::read_password;
use std::io::Write;
use std::io::Cursor;

use bsky_sdk::BskyAgent;
use atrium_api::types::string::Datetime;
use bsky_sdk::moderation::decision::DecisionContext;
use atrium_api::app::bsky::feed::get_timeline::ParametersData;

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



        match start_session(uname.trim().to_string(), pwd.to_string()).await {
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

async fn start_session(uname: String, pwd: String) -> Result<(), Box<dyn std::error::Error>> {
    let agent = BskyAgent::builder().build().await?;
    let session = agent.login(&uname,&pwd).await?;

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
    let mut cursor = None;
    loop {
        // returns a `GetTimelineResponse` whose `.feed: Vec<FeedViewPost>` has
        // `post.record` already deserialized into `app_bsky_feed_defs::FeedViewPostRecord`
        let resp = agent.get_timeline(cursor.clone(), Some(50)).await?;
        for item in resp.feed {
            // here `item.post.record` is a `FeedViewPostRecord` struct
            println!("— {}", item.post.record.text);
        }
        if let Some(next) = resp.cursor {
            cursor = Some(next);
        } else {
            break;
        }
    }

    Ok(())
}
