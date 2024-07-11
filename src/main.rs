use chrono::prelude::*;
use dotenv::dotenv;
use reqwest::Client;
use serde_json::json;
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
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    let project = env::var("SCRAPBOX_PROJECT_NAME").expect("SCRAPBOX_PROJECT_NAME must be set in .env file");

    println!("Writing to Scrapbox: {}...", title);

    if check_page_exists(&sid, &project, &title).await? {
        println!("Page \"{}\" already exists.", title);
        return Ok(());
    }

    write_to_scrapbox(&sid, &project, &title, &text).await?;

    println!("Done!");
    Ok(())
}

async fn check_page_exists(sid: &str, project: &str, title: &str) -> Result<bool, reqwest::Error> {
    let client = Client::new();
    let url = format!("https://scrapbox.io/api/pages/{}/{}/text", project, title);
    
    let response = client.get(&url)
        .header("Cookie", format!("connect.sid={}", sid))
        .send()
        .await?;

    Ok(response.status().is_success())
}

async fn write_to_scrapbox(sid: &str, project: &str, title: &str, text: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let url = format!("https://scrapbox.io/api/pages/{}", project);

    let response = client.post(&url)
        .header("Content-Type", "application/json")
        .header("Cookie", format!("connect.sid={}", sid))
        .json(&json!({
            "title": title,
            "content": text
        }))
        .send()
        .await?;

    if response.status().is_success() {
        println!("ページが正常に作成されました！");
    } else {
        println!("ページの作成に失敗しました。ステータスコード: {:?}", response.status());
        println!("{:?}", response.text().await?);
    }

    Ok(())
}

fn generate_daily_title() -> String {
    let local: DateTime<Local> = Local::now();
    let day = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"][local.weekday().num_days_from_sunday() as usize];
    format!("{}/{}/{} ({})", local.year(), local.month(), local.day(), day)
}

fn generate_weekly_title() -> String {
    let local: DateTime<Local> = Local::now();
    let end_date = local + chrono::Duration::days(6);
    format!("{}/{}/{} ~ {}/{}/{}", local.year(), local.month(), local.day(), end_date.year(), end_date.month(), end_date.day())
}