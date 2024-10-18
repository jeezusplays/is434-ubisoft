use clap::Parser;
use serde::Deserialize;
use serde_json::Value;
use std::io::Write;
use std::{collections::HashMap, fs::File};

#[derive(Parser)]
pub struct AppConfig {
    /// The app ids = game ids found on steam store, separated by commas
    #[clap(env, use_value_delimiter = true, value_delimiter = ',')]
    pub app_ids: Vec<String>,
}

#[derive(Clone, Deserialize, Debug)]
struct ReviewResponse {
    success: u32,
    query_summary: QuerySummary,
    reviews: Vec<Review>,
    cursor: String,
}

#[derive(Clone, Deserialize, Debug)]
struct QuerySummary {
    num_reviews: Option<u32>,
    _review_score: Option<u32>, // Change to Option to handle missing field
    _review_score_desc: Option<String>,
    _total_positive: Option<u32>,
    _total_negative: Option<u32>,
    _total_reviews: Option<u32>,
}

#[derive(Clone, Deserialize, Debug)]
struct Review {
    recommendationid: String,
    author: Author,
    language: String,
    review: String,
    timestamp_created: u64,
    timestamp_updated: u64,
    voted_up: bool,
    votes_up: u32,
    votes_funny: u32,
    weighted_vote_score: Value,
    comment_count: u32,
    steam_purchase: bool,
    received_for_free: bool,
    written_during_early_access: bool,
    _hidden_in_steam_china: bool,
    steam_china_location: String,
    _primarily_steam_deck: bool,
}

#[derive(Clone, Deserialize, Debug)]
struct Author {
    steamid: String,
    _num_games_owned: u32,
    _num_reviews: u32,
    playtime_forever: u32,
    playtime_last_two_weeks: u32,
    playtime_at_review: u32,
    _last_played: u64,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let config = AppConfig::parse();
    for app_id in config.app_ids {
        let mut cursor = "*".to_string();
        let mut reviews: Vec<Review> = Vec::new();
        let base_url = "https://store.steampowered.com/appreviews/";

        loop {
            // Set up the query parameters
            let mut params = HashMap::new();
            params.insert("json", "1");
            params.insert("filter", "recent");
            params.insert("language", "english");
            params.insert("day_range", "all");
            params.insert("cursor", &cursor);
            params.insert("review_type", "all");
            params.insert("purchase_type", "steam");
            params.insert("num_per_page", "100");

            // Make the request
            let url = format!("{}{}", base_url, app_id);
            let client = reqwest::Client::new();
            let result = client.get(&url).query(&params).send().await;
            if result.is_err() {
                println!("Error: {}", result.err().unwrap());
                continue;
            }

            let response = result.unwrap();
            let res: ReviewResponse = response.json().await?;
            if res.success == 0 {
                println!("Error: {}", res.success);
                continue;
            }

            // Append reviews to the list
            reviews.extend(res.reviews.clone());
            println!(
                "Current number of reviews fetched for {}: {}",
                app_id,
                reviews.len()
            );

            // Check if we have retrieved all reviews
            if res.query_summary.num_reviews.is_some()
                && res.query_summary.num_reviews.unwrap() < 100
                || cursor == res.cursor
            {
                break;
            }

            // Update the cursor for the next page
            cursor = res.cursor.clone();
        }

        // Print the total number of reviews fetched
        println!("Total reviews fetched: {}", reviews.len());

        // Process the reviews (you can print, store, etc.)
        // Create a CSV file
        let mut file = File::create(format!("reviews_{}.csv", app_id))?;

        // Write CSV header
        writeln!(file, "recommendationid,steamid,timestamp_created,playtime_forever,review,voted_up,votes_up,votes_funny,weighted_vote_score,comment_count,steam_purchase,received_for_free,written_during_early_access,timestamp_updated,playtime_at_review,playtime_last_two_weeks,language,steam_china_location")?;

        // Write each review as a CSV row
        for review in &reviews {
            writeln!(
                file,
                "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                review.recommendationid,
                review.author.steamid,
                review.timestamp_created,
                review.author.playtime_forever,
                review.review.replace(",", "").replace("\n", " "), // Escape commas and newlines
                review.voted_up,
                review.votes_up,
                review.votes_funny,
                review.weighted_vote_score,
                review.comment_count,
                review.steam_purchase,
                review.received_for_free,
                review.written_during_early_access,
                review.timestamp_updated,
                review.author.playtime_at_review,
                review.author.playtime_last_two_weeks,
                review.language,
                review.steam_china_location
            )?;
        }
        println!("Reviews exported");
    }

    Ok(())
}
