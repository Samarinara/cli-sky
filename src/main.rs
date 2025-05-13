use std::io;

use bsky_sdk::BskyAgent;
use atrium_api::types::string::Datetime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}[2J", 27 as char);
    println!(" /$$$$$$  /$$       /$$$$$$        /$$$$$$  /$$   /$$ /$$     /$$");
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
        println!("Password:");

        let mut pwd = String::new();
        std::io::stdin().read_line(&mut pwd).expect("Failed to read");

        match start_session(uname.trim().to_string(), pwd.trim().to_string()).await {
            Ok(_) => {
                // Session started successfully
                return Ok(());
            }
            Err(e) => {
                println!("\nLogin failed. Please check your handle and password.");
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

//    print!("{}[2J", 27 as char);


    println!("What would you like to do?");
    println!("1: Text Post");
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
