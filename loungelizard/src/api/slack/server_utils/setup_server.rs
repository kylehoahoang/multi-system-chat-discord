
use serde::Deserialize;
use slack_morphism::prelude::*;
use std::fs;
use url::Url;
use crate::api::slack::ngrok_s::*;
use dioxus_logger::tracing::{info, error, warn};
use crate::api::mongo_format::mongo_structs::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use reqwest::header::{CONTENT_TYPE, CONTENT_LENGTH, HOST};
use reqwest::{Client as ReqwestClient, Response};
use std::collections::HashMap;


pub async fn start_endpoint() -> Result<SlackAppManifest, Box<dyn std::error::Error>>
{
    // Define the path to the JSON file
    let file_path = "src/api/slack/manifest/manifest.json";

    // Read the contents of the manifest file
    let manifest_file = fs::read_to_string(file_path).expect("Unable to read file");

    // Parse the manifest file into a `SlackAppManifest` struct
    let mut manifest_struct: SlackAppManifest = serde_json::from_str(&manifest_file).expect("Unable to parse JSON");

    // Try to fetch ngrok tunnels
    let response = match fetch_ngrok_tunnels().await {
        Ok(response) => {
            info!("Fetched ngrok tunnels successfully.");
            response
        },
        Err(err) => {
            info!("Failed to fetch ngrok tunnels: {}. Attempting to start ngrok session...", err);

            // Attempt to start the ngrok session
            match ngrok_start_session("8080") {
                Ok(_child) => {
                    info!("ngrok session started successfully.");

                    // Wait for a brief period to allow ngrok to start
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

                    // Retry fetching ngrok tunnels
                    match fetch_ngrok_tunnels().await {
                        Ok(response) => {
                            info!("Successfully fetched ngrok tunnels after starting session.");
                            response // Return this response
                        },
                        Err(err) => {
                            // If we fail again, log and return the error
                            info!("Failed to fetch ngrok tunnels after starting session: {}", err);
                            return Err(Box::new(err)); // Return the error wrapped in Box
                        }
                    }
                },
                Err(start_err) => {
                    info!("Failed to start ngrok session: {}", start_err);
                    return Err(Box::new(start_err)); // Return the error wrapped in Box
                }
            }
        }
    };

    let mut redirect_url = String::new();
    // Extract the public URL from the first active tunnel
    if let Some(tunnels) = response.get("tunnels").and_then(|t| t.as_array()) {
        if let Some(tunnel) = tunnels.first() {
            if let Some(public_url) = tunnel.get("public_url").and_then(|url| url.as_str()) {
                // Set the environment variable `SLACK_REDIRECT_HOST` to the public URL
                redirect_url = public_url.to_string();
            } else {
                eprintln!("Public URL not found in the tunnel data.");
            }
        } else {
            eprintln!("No tunnels found.");
        }
    } else {
        eprintln!("Tunnels field not found in the response.");
    }

    // Update newly created URL into manifest befor creating the app
    if let Some(ref mut settings) = manifest_struct.settings {
        if let Some(ref mut event_subscriptions) = settings.event_subscriptions {
            if let Some(ref mut request_url) = event_subscriptions.request_url {
                // Update the request URL to the public URL
                if !redirect_url.is_empty() {
                    *request_url = Url::parse(redirect_url.as_str()).expect("Failed to parse URL");
                }
                else {
                    error!("Failed to parse URL into manifest");
                }
                
            }
        }
    }

    if let Some(ref mut oauth_config) = manifest_struct.oauth_config {
        if let Some(ref mut redir_urls) = oauth_config. redirect_urls{
            // Update the redirect URLs to contain only the public URL
            if !redirect_url.is_empty() {
                *redir_urls = vec![(Url::parse(redirect_url.as_str()).expect("Failed to parse URL"))];
            }
            else {
                error!("Failed to parse URL into manifest");
            }
        }
    }

    Ok(manifest_struct)
   
}


pub async fn update_slack_app(
    user: User 
) -> Result<ResponseData, Box<dyn std::error::Error>> 
{
    // Create a new Slack client 
    let client  = SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    let user_temp = user.clone(); 

    let user_refresh_token = user_temp.clone().slack.refresh_token; 
    let mut form_data = HashMap::new();
    form_data.insert("refresh_token", user_refresh_token.as_str());

    let client_r = ReqwestClient::new();

    let response = 
        match client_r
                .post("https://slack.com/api/tooling.tokens.rotate")
                .header(HOST, "slack.com")
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .form(&form_data)
                .send()
                .await
        {
            Ok(response) => response,
            Err(err) => {
                warn!("Error Encountered {}", err);
                return Err(Box::new(err) as Box<dyn std::error::Error>);
            }
        };

    let mut response_data_exp: ResponseData = ResponseData::default();
    let mut new_access_token = String::new(); 
    // Check if response is successful
    if response.status().is_success() {
        // Try to print the raw response before parsing
        let raw_body = response.text().await?;

        // Attempt to parse the response body as JSON
        match serde_json::from_str::<ResponseData>(&raw_body) {
            Ok(response_data) => {
                new_access_token = response_data.token;
                // Attempt to parse the response body as JSON
                response_data_exp = serde_json::from_str(&raw_body)?;
            }
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
            }
        }
    } else {
        println!("Failed to get a successful response. Status: {}", response.status());
    }

    let token: SlackApiToken = SlackApiToken::new(new_access_token.into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    // Start server to generate a app manifest structure 
    let manifest_struct = match start_endpoint().await
    {
        Ok(manifest) => manifest,
        Err(err) => return Err(err),
    };

    // Update existing app
    let updated_app = SlackApiAppsManifestUpdateRequest::new(
        user.slack.app_id.clone().into(),
        manifest_struct.clone()
    );

    return match session.apps_manifest_update(&updated_app).await {
        Ok(_response) => Ok(response_data_exp),
        Err(err) => Err(Box::new(err) as Box<dyn std::error::Error>),
    };


}


pub async fn create_slack_app(
    user_lock: Arc<Mutex<User>>,
    refresh_token: String,
) -> Result<(), Box<dyn std::error::Error>> 
{
    let mut user = user_lock.lock().await; 
    
    let refresh_token = match refresh_token.starts_with("xoxe.xoxp") {
        true => {
            // Ensure Token is not an access token
            return Err("Refresh Token needed".into());
        },
        false => {
            // If the token does not start with the correct prefix, return an error
            match refresh_token.starts_with("xoxe"){
                // Return Access token if it starts with xoxe
                true => {refresh_token},
                false => {
                    match refresh_token.as_str() {
                        "\n" => {
                            // If the token is empty, return an error
                            return Err("Nothing in Clipboard".into());
                        },
                        _ => {
                            // If the token is empty, return an error
                            return Err("Incorrect token".into());
                    }
                    }
                }
            } 
            
        }
    };
    let mut form_data = HashMap::new();
    form_data.insert("refresh_token", refresh_token.as_str());

    let client_r = ReqwestClient::new();

    let response = 
        match client_r
                .post("https://slack.com/api/tooling.tokens.rotate")
                .header(HOST, "slack.com")
                .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .form(&form_data)
                .send()
                .await
        {
            Ok(response) => response,
            Err(err) => {
                warn!("Error Encountered {}", err);
                return Err(Box::new(err) as Box<dyn std::error::Error>);
            }
        };

    let mut response_data_exp: ResponseData = ResponseData::default();
    let mut new_access_token = String::new(); 
    // Check if response is successful
    if response.status().is_success() {
        // Try to print the raw response before parsing
        let raw_body = response.text().await?;

        // Attempt to parse the response body as JSON
        match serde_json::from_str::<ResponseData>(&raw_body) {
            Ok(response_data) => {
                new_access_token = response_data.token;
                // Attempt to parse the response body as JSON
                response_data_exp = serde_json::from_str(&raw_body)?;
            }
            Err(e) => {
                error!("Failed to parse JSON: {}", e);
            }
        }
    } else {
        println!("Failed to get a successful response. Status: {}", response.status());
    }

    // Create a new Slack client 
    let client  = SlackClient::new(SlackClientHyperConnector::new().expect("failed to create hyper connector"));

    // Create a new token using config_token
    let token: SlackApiToken = SlackApiToken::new(new_access_token.clone().into());

    // Create a new session with the client and the token
    let session = client.open_session(&token);

    // Start server to generate a app manifest structure 
    let manifest_struct = match start_endpoint().await
    {
        Ok(manifest) => manifest,
        Err(err) => return Err(err),
    };

    // Create a new app with the updated manifest
    let new_app: SlackApiAppsManifestCreateRequest = SlackApiAppsManifestCreateRequest::new(
        SlackAppId::new("-".into()),
        manifest_struct.clone()
    );

    // Create the app
    return match session.apps_manifest_create(&new_app).await
    {
        Ok(response) => {
            // Set Env vars without manually inputting them 
            user.slack.client_id = response.credentials.client_id.to_string();
            user.slack.client_secret = response.credentials.client_secret.to_string();
            user.slack.verif_token = response.credentials.verification_token.to_string();
            user.slack.oauth_url = response.oauth_authorize_url.to_string();
            user.slack.config_token = new_access_token;
            user.slack.refresh_token = refresh_token;
            Ok(())
        }
        Err(err) => Err(Box::new(err) as Box<dyn std::error::Error>),
    }

}