use dioxus::prelude::*;
use bson::to_bson;
use crate::{MONGO_COLLECTION, MONGO_DATABASE};

use crate::api::mongo_format::mongo_structs::*;
use crate::api::ms_teams::ms_teams_app_setup::start_ms_teams;
use dioxus_logger::tracing::{info, error, warn};
use futures::executor::block_on;
use mongodb::{sync::Client, bson::doc};

use std::sync::Arc;
use tokio::sync::Mutex;

const MS_TEAMS_CLIENT_ID: &str = "51e0dbc4-59a4-4cb4-a020-1b8ef7495470";

#[component]
pub fn MSTeamsLogin (
    show_teams_login_pane: Signal<bool>,
    show_teams_server_pane: Signal<bool>,
    current_platform: Signal<String>,
    access_token: Signal<String>,
    refresh_token: Signal<String>,
    expiration: Signal<String>
) -> Element {

   // ! User Mutex Lock to access the user data
   let user_lock = use_context::<Signal<Arc<Mutex<User>>>>();
   let client_lock = use_context::<Signal<Arc<Mutex<Option<Client>>>>>();
   // ! ========================= ! //
    let mut logged_in = use_signal(|| false);
    let mut login_error = use_signal(|| None::<String>);

    let handle_login = move |_| {

        block_on(async move {
            match start_ms_teams(MS_TEAMS_CLIENT_ID).await {
                Ok(token) => {
                    access_token.set(token.0);
                    refresh_token.set(token.1);
                    expiration.set(token.2);
                    show_teams_login_pane.set(false);
                    show_teams_server_pane.set(true);
                    info!("Login successful");
                }
                Err(e) => {
                    login_error.set(Some(e.to_string()));
                    info!("Login failed: {}", e);
                }
            }
        });


        // TODO: ========================
        // If there is an issue, prevent the login from happening
        if !login_error().is_none() {return;}

        let mongo_lock_copies = client_lock().clone();
        let user_lock_copies = user_lock().clone();

        let mongo_client = block_on(
            async {
                mongo_lock_copies.lock().await
            }
        );

        let mut user = block_on(
            async {
                user_lock_copies.lock().await
            }
        );
         // Clone the client if it exists (since we can't return a reference directly)
        if let Some(client) = mongo_client.as_ref() {
            // Convert the function into async and spawn it on the current runtime
            let client_clone = client.clone();  // Clone the client to avoid ownership issues

            // TODO Add all tokens to user profile here
            user.ms_teams = MSTeams{
                access_token: access_token.to_string(),
                refresh_token: refresh_token.to_string(),
                expiration: expiration.to_string()

            };
            
            // Todo ====================================//

            let user_clone = user.clone();
            
            // Use `tokio::spawn` to run the async block
            spawn(async move {
                let db = client_clone.database(MONGO_DATABASE);
                let user_collection = db.collection::<User>(MONGO_COLLECTION);
                
                match to_bson(&user_clone.ms_teams) {
                    Ok(ms_teams_bson) => {
                        match user_collection
                            .find_one_and_update(
                                doc! { 
                                    "$or": [{"username": &user_clone.username}, 
                                            {"email": &user_clone.email}] 
                                },
                                doc! {
                                    "$set": { "ms_teams": ms_teams_bson }
                                }
                            )
                            .await 
                        {
                            Ok(Some(_)) => {
                                // Document found and updated
                                info!("Document updated successfully");
                                logged_in.set(true);
                                current_platform.set("MSTeams".to_string());
                            }
                            Ok(None) => {
                                // No document matched the filter
                                warn!("Document not found");
                            }
                            Err(e) => {
                                error!("Something went wrong: {:#?}", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to convert Slack to BSON: {:#?}", e);
                    }
                }
                
            });

        } else {
            warn!("MongoDB client not found in global state.");
        }

    };

    rsx! {
        div {
            class: format_args!("teams-login {}", if show_teams_login_pane() { "visible" } else { "" }),
            img {
                src: "assets/msteams_logo.png",
                alt: "MSTeams Logo",
                width: "50px",
                height: "50px",
            }
            button {
                style: "position: absolute; top: 10px; right: 10px; background-color: transparent; border: none; cursor: pointer;",
                onclick: move |_| { show_teams_login_pane.set(false) },
                svg {
                    xmlns: "http://www.w3.org/2000/svg",
                    view_box: "0 0 24 24",
                    width: "30", // Adjust size as needed
                    height: "30", // Adjust size as needed
                    path {
                        d: "M18 6 L6 18 M6 6 L18 18", // This path describes a close icon (X)
                        fill: "none",
                        stroke: "#f5f5f5", // Change stroke color as needed
                        stroke_width: "2" // Adjust stroke width
                    }
                }
            }
            button { 
                class: "login-button",
                onclick: handle_login, "Login" 
            }

            // TODO: provide custom error warnings
            if let Some(error) = login_error() {
                p { 
                    style: "color: white; font-family: Arial, sans-serif; font-weight: bold; text-align: center;",
                    "Login failed: {error}" 
                }
            }
        }
    }
}