use anyhow::Result;
use chrono::prelude::*;
use dotenv::dotenv;
use headless_chrome::Browser;
use std::env;
use structopt::StructOpt;

const DAILY_TEXT: &str = "[* ルーティン]\n[* 感想]\n#daily";
const WEEKLY_TEXT: &str = "[* 目標]\n[* 振り返り]\n[* 感想]\n[* 日記]\n#weekly";

#[derive(StructOpt)]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt)]
enum Command {
    Daily,
    Weekly,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let args = Cli::from_args();
    let (title, text) = match args.cmd {
        Command::Daily => {
            let title = generate_daily_title();
            (title, DAILY_TEXT.to_string())
        }
        Command::Weekly => {
            let title = generate_weekly_title();
            (title, WEEKLY_TEXT.to_string())
        }
    };

    let sid = env::var("SCRAPBOX_SID").expect("SCRAPBOX_SID must be set in .env file");
    let project =
        env::var("SCRAPBOX_PROJECT_NAME").expect("SCRAPBOX_PROJECT_NAME must be set in .env file");

    println!("Writing to Scrapbox: {}...", title);

    let browser = Browser::default()?;
    println!("Browser launched successfully.");
    let tab = browser.new_tab()?;
    println!("Tab created successfully.");

    // Set cookie
    tab.set_cookies(vec![headless_chrome::protocol::cdp::Network::CookieParam {
        name: "connect.sid".to_string(),
        value: sid,
        url: Some("https://scrapbox.io/".to_string()),
        domain: Some("scrapbox.io".to_string()),
        path: Some("/".to_string()),
        expires: None,
        http_only: Some(true),
        secure: Some(true),
        same_site: None,
        priority: Some(headless_chrome::protocol::cdp::Network::CookiePriority::Medium),
        same_party: Some(false),
        source_scheme: Some(headless_chrome::protocol::cdp::Network::CookieSourceScheme::Secure),
        source_port: Some(443),
        partition_key: None,
    }])?;

    // Check if page exists
    let url = format!("https://scrapbox.io/{}/{}", project, title);
    tab.navigate_to(&url)?;
    tab.wait_for_element("body")?;

    let page_exists = tab
        .evaluate(
            "!document.body.innerText.includes('create new page')",
            false,
        )?
        .value
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if page_exists {
        println!("Page \"{}\" already exists.", title);
        return Ok(());
    }

    // Create new page
    let create_url = format!(
        "https://scrapbox.io/{}/{}?body={}",
        project,
        title,
        urlencoding::encode(&text)
    );
    tab.navigate_to(&create_url)?;
    tab.wait_for_element("body")?;

    // Wait for page to be created
    tab.wait_for_element(".page-wrapper")?;

    println!("Page created successfully!");

    Ok(())
}

fn generate_daily_title() -> String {
    let local: DateTime<Local> = Local::now();
    let day = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"]
        [local.weekday().num_days_from_sunday() as usize];
    format!(
        "{}/{}/{} ({})",
        local.year(),
        local.month(),
        local.day(),
        day
    )
}

fn generate_weekly_title() -> String {
    let local: DateTime<Local> = Local::now();
    let end_date = local + chrono::Duration::days(6);
    format!(
        "{}/{}/{} ~ {}/{}/{}",
        local.year(),
        local.month(),
        local.day(),
        end_date.year(),
        end_date.month(),
        end_date.day()
    )
}
