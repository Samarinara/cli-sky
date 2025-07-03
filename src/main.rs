use std::fs::File;
use std::io::Write;
use std::process;
use std::fs;
use std::io::{self, BufRead};
use std::str::FromStr;

use keyring::error::Error;
use bsky_sdk::BskyAgent;
use atrium_api::types::string::Datetime;
use atrium_api::app::bsky::feed::get_timeline::ParametersData;
use bsky_sdk::agent::config::{Config, FileStore};
use atrium_api::app::bsky::feed::post::RecordData as PostRecordData;
use serde_json::from_value;
use serde::{Deserialize, Serialize};

use atrium_api::com::atproto::repo::create_record::InputData;


async fn print_logo() {
    println!("{}[2J", 27 as char);
    println!("  /██████  /██       /██████        /██████  /██   /██ /██     /██");
    println!(" /██__  ██| ██      |_  ██_/       /██__  ██| ██  /██/|  ██   /██/");
    println!("| ██  |__/| ██        | ██        | ██  |__/| ██ /██/  |  ██ /██/ ");
    println!("| ██      | ██        | ██        |  ██████ | █████/    |  ████/  ");
    println!("| ██      | ██        | ██         |____  ██| ██  ██     |  ██/   ");
    println!("| ██    ██| ██        | ██         /██  | ██| ██|  ██     | ██    ");
    println!("|  ██████/| ████████ /██████      |  ██████/| ██ |  ██    | ██    ");
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
                Ok(4) => {
                    println!("find a blog");
                    list_user_blog(&agent).await?;
                    break;
                }
                Ok(5) => {
                    println!("find a blog (experimental)");
                    list_user_blog_experimental(&agent).await?;
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
        .api.app.bsky.feed.get_timeline(
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


pub async fn list_user_blog(agent: &BskyAgent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter blogger's handle: ");
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let handle = name.trim();
    println!("Looking up handle: {}", handle);

    // Get the author's DID
    let did = match agent.api.com.atproto.identity.resolve_handle(
        atrium_api::com::atproto::identity::resolve_handle::ParametersData {
            handle: atrium_api::types::string::Handle::from_str(handle)?,
        }
        .into(),
    ).await {
        Ok(response) => {
            println!("Successfully resolved handle to DID: {:?}", response.did);
            response.did.clone()
        },
        Err(e) => {
            println!("Error: Could not find user with handle '{}'. Please check the handle and try again.", handle);
            println!("Error details: {}", e);
            menu(agent.clone()).await?;
            return Ok(());
        }
    };

    println!("Fetching blog posts for DID: {:?}", did);

    // Get the blog posts using listRecords
    let params = atrium_api::com::atproto::repo::list_records::ParametersData {
        repo: atrium_api::types::string::AtIdentifier::from_str(&format!("at://{}", did.to_string()))?,
        collection: "com.macroblog.blog.post".parse()?,
        limit: Some(50.try_into()?),
        cursor: None,
        reverse: None,
    };
    
    println!("List records parameters: {:?}", params);

    let response = match agent
        .api
        .com
        .atproto
        .repo
        .list_records(params.into())
        .await {
            Ok(response) => {
                println!("Successfully fetched records. Number of records: {}", response.records.len());
                if response.records.is_empty() {
                    println!("No records found in response");
                } else {
                    let record_value = serde_json::to_value::<atrium_api::types::Unknown>(response.records[0].value.clone())?;
                    println!("First record value: {:?}", record_value);
                }
                response
            },
            Err(e) => {
                println!("Error: Could not fetch blog posts. The user may not have any blog posts or they may be private.");
                println!("Error details: {}", e);
                println!("Error type: {:?}", e);
                println!("DID used: {:?}", did);
                println!("Collection: com.macroblog.blog.post");
                println!("Full repository identifier: at://{}", did.to_string());
                println!("Debug info - DID string: {}", did.to_string());
                println!("Debug info - Collection string: com.macroblog.blog.post");
                println!("Debug info - Raw error: {:?}", e);
                return Ok(());
            }
        };

    if response.records.is_empty() {
        println!("No blog posts found for user '{}'.", handle);
        menu(agent.clone()).await?;
        return Ok(());
    }

    // Print out blog posts
    for (i, record) in response.records.iter().enumerate() {
        println!("\nProcessing record {} of {}", i + 1, response.records.len());
        let record_value = serde_json::to_value::<atrium_api::types::Unknown>(record.value.clone())?;
        
        // Debug print the record type
        println!("Record value: {:?}", record_value);
        
        // Parse the blog post
        match serde_json::from_value::<BlogPost>(record_value.clone()) {
            Ok(blog_post) => {
                println!("\n{}", "=".repeat(50));
                println!(
                    "Title: {}\n\n{}",
                    blog_post.title,
                    blog_post.text
                );
                if let Some(tags) = blog_post.tags {
                    println!("\nTags: {}", tags.join(", "));
                }
                println!("{}", "=".repeat(50));
            }
            Err(e) => {
                println!("Error parsing blog post: {}", e);
                println!("Raw record value: {:?}", record_value);
                continue;
            }
        }
        println!("\nPress Enter to continue to next post...");
        let mut dummy = String::new();
        io::stdin().read_line(&mut dummy)?;
    }

    println!("\nPress Enter to return to menu...");
    let mut dummy = String::new();
    io::stdin().read_line(&mut dummy)?;  
    menu(agent.clone()).await?;  
    Ok(())
}

pub async fn list_user_blog_experimental(agent: &BskyAgent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter blogger's handle: ");
    io::stdout().flush()?;

    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    let handle = name.trim();
    println!("Looking up handle: {}", handle);

    // Get the author's DID
    let did = match agent.api.com.atproto.identity.resolve_handle(
        atrium_api::com::atproto::identity::resolve_handle::ParametersData {
            handle: atrium_api::types::string::Handle::from_str(handle)?,
        }
        .into(),
    ).await {
        Ok(response) => {
            println!("Successfully resolved handle to DID: {:?}", response.did);
            response.did.clone()
        },
        Err(e) => {
            println!("Error: Could not find user with handle '{}'. Please check the handle and try again.", handle);
            println!("Error details: {}", e);
            return Ok(());
        }
    };

    println!("Fetching posts for DID: {:?}", did);

    // First try to get the user's profile to verify access
    match agent.api.app.bsky.actor.get_profile(
        atrium_api::app::bsky::actor::get_profile::ParametersData {
            actor: atrium_api::types::string::AtIdentifier::from_str(&did.to_string())?,
        }
        .into(),
    ).await {
        Ok(profile) => {
            println!("Successfully accessed profile for: {}", profile.display_name.as_ref().unwrap_or(&"Unknown".to_string()));
        },
        Err(e) => {
            println!("Warning: Could not access profile. This might indicate the account is private or suspended.");
            println!("Profile error details: {}", e);
            return Ok(());
        }
    }

    // Try to get the author's feed using the feed API first
    println!("\nAttempting to fetch author feed...");
    let feed_params = atrium_api::app::bsky::feed::get_author_feed::ParametersData {
        actor: atrium_api::types::string::AtIdentifier::from_str(&did.to_string())?,
        cursor: None,
        filter: None, // Try without filter first
        limit: Some(50.try_into()?),
        include_pins: None,
    };

    println!("Feed parameters: {:?}", feed_params);

    match agent
        .api
        .app
        .bsky
        .feed
        .get_author_feed(feed_params.into())
        .await {
            Ok(feed_response) => {
                println!("Successfully fetched feed. Number of posts: {}", feed_response.feed.len());
                println!("Feed response: {:?}", feed_response);
                
                if feed_response.feed.is_empty() {
                    println!("No posts found in feed, trying repository API...");
                    
                    // Try repository API as fallback
                    let params = atrium_api::com::atproto::repo::list_records::ParametersData {
                        repo: atrium_api::types::string::AtIdentifier::from_str(&format!("at://{}", did.to_string()))?,
                        collection: "app.bsky.feed.post".parse()?,
                        limit: Some(100.try_into()?),
                        cursor: None,
                        reverse: Some(true), // Try getting most recent posts first
                    };
                    
                    println!("List records parameters: {:?}", params);

                    match agent
                        .api
                        .com
                        .atproto
                        .repo
                        .list_records(params.into())
                        .await {
                            Ok(response) => {
                                println!("Successfully fetched records. Number of records: {}", response.records.len());
                                println!("Records response: {:?}", response);
                                
                                if response.records.is_empty() {
                                    println!("No posts found for user '{}'.", handle);
                                    return Ok(());
                                }

                                // Print out records
                                for (i, record) in response.records.iter().enumerate() {
                                    println!("\nProcessing record {} of {}", i + 1, response.records.len());
                                    
                                    // Convert the record value to a JSON value for easier access
                                    let record_value = match serde_json::to_value(&record.value) {
                                        Ok(value) => value,
                                        Err(e) => {
                                            println!("Error converting record to JSON: {}", e);
                                            continue;
                                        }
                                    };

                                    // Get the record type
                                    let record_type = record_value.get("$type").and_then(|t| t.as_str()).unwrap_or("unknown");
                                    println!("Record type: {}", record_type);

                                    // Try to parse as a post record
                                    match serde_json::from_value::<PostRecordData>(record_value.clone()) {
                                        Ok(post) => {
                                            println!("\n{}", "=".repeat(50));
                                            println!("Text: {}", post.text);
                                            
                                            // Print any embedded content
                                            if let Some(embed) = post.embed {
                                                println!("\nEmbedded content:");
                                                println!("{:?}", embed);
                                            }
                                            
                                            // Print any facets
                                            if let Some(facets) = post.facets {
                                                println!("\nFacets:");
                                                for facet in facets {
                                                    println!("{:?}", facet);
                                                }
                                            }
                                            
                                            // Print any tags
                                            if let Some(tags) = post.tags {
                                                println!("\nTags: {}", tags.join(", "));
                                            }
                                            
                                            println!("{}", "=".repeat(50));
                                        },
                                        Err(e) => {
                                            println!("Could not parse as post record: {}", e);
                                            // Fall back to displaying raw text if available
                                            if let Some(text) = record_value.get("text").and_then(|t| t.as_str()) {
                                                println!("\n{}", "=".repeat(50));
                                                println!("Text: {}", text);
                                                println!("{}", "=".repeat(50));
                                            }
                                        }
                                    }
                                    
                                    println!("\nPress Enter to continue to next post...");
                                    let mut dummy = String::new();
                                    io::stdin().read_line(&mut dummy)?;
                                }
                            },
                            Err(e) => {
                                println!("Error: Could not fetch records.");
                                println!("Error details: {}", e);
                                println!("Error type: {:?}", e);
                                println!("DID used: {:?}", did);
                                println!("Collection: app.bsky.feed.post");
                                println!("Full repository identifier: at://{}", did.to_string());
                                println!("Debug info - DID string: {}", did.to_string());
                                println!("Debug info - Raw error: {:?}", e);
                                
                                // Try one more time with a different collection
                                println!("\nTrying with different collection...");
                                let params = atrium_api::com::atproto::repo::list_records::ParametersData {
                                    repo: atrium_api::types::string::AtIdentifier::from_str(&format!("at://{}", did.to_string()))?,
                                    collection: "com.atproto.repo.strongRef".parse()?,
                                    limit: Some(100.try_into()?),
                                    cursor: None,
                                    reverse: Some(true),
                                };
                                
                                println!("List records parameters (second attempt): {:?}", params);
                                
                                match agent
                                    .api
                                    .com
                                    .atproto
                                    .repo
                                    .list_records(params.into())
                                    .await {
                                        Ok(response) => {
                                            println!("Successfully fetched records (second attempt). Number of records: {}", response.records.len());
                                            println!("Records response (second attempt): {:?}", response);
                                        },
                                        Err(e) => {
                                            println!("Error in second attempt: {}", e);
                                            println!("Error type: {:?}", e);
                                            
                                            // Try one last time with the relay service
                                            println!("\nTrying with relay service...");
                                            let params = atrium_api::com::atproto::sync::get_repo::ParametersData {
                                                did: did.clone(),
                                                since: None,
                                            };
                                            
                                            println!("Get repo parameters: {:?}", params);
                                            
                                            match agent
                                                .api
                                                .com
                                                .atproto
                                                .sync
                                                .get_repo(params.into())
                                                .await {
                                                    Ok(response) => {
                                                        println!("Successfully fetched repo from relay service");
                                                        println!("Repo response: {:?}", response);
                                                    },
                                                    Err(e) => {
                                                        println!("Error fetching from relay service: {}", e);
                                                        println!("Error type: {:?}", e);
                                                    }
                                                }
                                        }
                                    }
                                
                                return Ok(());
                            }
                        }
                } else {
                    // Print out feed posts
                    for (i, feed_view) in feed_response.feed.iter().enumerate() {
                        println!("\nProcessing post {} of {}", i + 1, feed_response.feed.len());
                        
                        // Get the record value and print it directly
                        let record_value = serde_json::to_value::<atrium_api::types::Unknown>(feed_view.post.record.clone())?;
                        println!("\n{}", "=".repeat(50));
                        println!("Record type: {:?}", record_value.get("$type"));
                        
                        // Try to get the text field directly
                        if let Some(text) = record_value.get("text").and_then(|t| t.as_str()) {
                            println!("Text: {}", text);
                        }
                        
                        // Try to get the title field if it exists
                        if let Some(title) = record_value.get("title").and_then(|t| t.as_str()) {
                            println!("Title: {}", title);
                        }
                        
                        // Try to get tags if they exist
                        if let Some(tags) = record_value.get("tags").and_then(|t| t.as_array()) {
                            let tag_strings: Vec<String> = tags.iter()
                                .filter_map(|tag| tag.as_str().map(String::from))
                                .collect();
                            if !tag_strings.is_empty() {
                                println!("Tags: {}", tag_strings.join(", "));
                            }
                        }
                        
                        println!("{}", "=".repeat(50));
                        println!("\nPress Enter to continue to next post...");
                        let mut dummy = String::new();
                        io::stdin().read_line(&mut dummy)?;
                    }
                }
            },
            Err(e) => {
                println!("Error fetching feed: {}", e);
                println!("Error type: {:?}", e);
                println!("Trying repository API as fallback...");
                
                // Fall back to repository API
                let params = atrium_api::com::atproto::repo::list_records::ParametersData {
                    repo: atrium_api::types::string::AtIdentifier::from_str(&format!("at://{}", did.to_string()))?,
                    collection: "app.bsky.feed.post".parse()?,
                    limit: Some(100.try_into()?),
                    cursor: None,
                    reverse: Some(true),
                };
                
                println!("List records parameters: {:?}", params);

                match agent
                    .api
                    .com
                    .atproto
                    .repo
                    .list_records(params.into())
                    .await {
                        Ok(response) => {
                            println!("Successfully fetched records. Number of records: {}", response.records.len());
                            println!("Records response: {:?}", response);
                            
                            if response.records.is_empty() {
                                println!("No posts found for user '{}'.", handle);
                                return Ok(());
                            }

                            // Print out records
                            for (i, record) in response.records.iter().enumerate() {
                                println!("\nProcessing record {} of {}", i + 1, response.records.len());
                                
                                // Convert the record value to a JSON value for easier access
                                let record_value = match serde_json::to_value(&record.value) {
                                    Ok(value) => value,
                                    Err(e) => {
                                        println!("Error converting record to JSON: {}", e);
                                        continue;
                                    }
                                };

                                // Get the record type
                                let record_type = record_value.get("$type").and_then(|t| t.as_str()).unwrap_or("unknown");
                                println!("Record type: {}", record_type);

                                // Try to parse as a post record
                                match serde_json::from_value::<PostRecordData>(record_value.clone()) {
                                    Ok(post) => {
                                        println!("\n{}", "=".repeat(50));
                                        println!("Text: {}", post.text);
                                        
                                        // Print any embedded content
                                        if let Some(embed) = post.embed {
                                            println!("\nEmbedded content:");
                                            println!("{:?}", embed);
                                        }
                                        
                                        // Print any facets
                                        if let Some(facets) = post.facets {
                                            println!("\nFacets:");
                                            for facet in facets {
                                                println!("{:?}", facet);
                                            }
                                        }
                                        
                                        // Print any tags
                                        if let Some(tags) = post.tags {
                                            println!("\nTags: {}", tags.join(", "));
                                        }
                                        
                                        println!("{}", "=".repeat(50));
                                    },
                                    Err(e) => {
                                        println!("Could not parse as post record: {}", e);
                                        // Fall back to displaying raw text if available
                                        if let Some(text) = record_value.get("text").and_then(|t| t.as_str()) {
                                            println!("\n{}", "=".repeat(50));
                                            println!("Text: {}", text);
                                            println!("{}", "=".repeat(50));
                                        }
                                    }
                                }
                                
                                println!("\nPress Enter to continue to next post...");
                                let mut dummy = String::new();
                                io::stdin().read_line(&mut dummy)?;
                            }
                        },
                        Err(e) => {
                            println!("Error: Could not fetch records.");
                            println!("Error details: {}", e);
                            println!("Error type: {:?}", e);
                            println!("DID used: {:?}", did);
                            println!("Collection: app.bsky.feed.post");
                            println!("Full repository identifier: at://{}", did.to_string());
                            println!("Debug info - DID string: {}", did.to_string());
                            println!("Debug info - Raw error: {:?}", e);
                            return Ok(());
                        }
                    }
            }
        }

    println!("\nPress Enter to return to menu...");
    let mut dummy = String::new();
    io::stdin().read_line(&mut dummy)?;  
    Ok(())
}
 