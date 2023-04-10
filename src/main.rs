mod db;

use uuid::{Uuid};
use serde::{Deserialize, Serialize};
use warp::{Filter, Reply, Rejection};
use mongodb::{error::Error, bson, bson::doc, Collection};

#[derive(Deserialize, Serialize)]
struct User {
    username: String,
    password: String
}

#[derive(Deserialize, Serialize)]
struct FullUser {
    username: String,
    password: String,
    id: Uuid
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let database = db::Database::new().await?;
    let users_collection = database.collection("users");

    let users_route = warp::path("users");

    let database_copy = database.clone();
    let users_collection_copy = users_collection.clone();

    let create_user_route = warp::post()
        .and(users_route.clone())
        .and(warp::any().map(move || database.clone()))
        .and(warp::any().map(move || users_collection.clone()))
        .and(warp::body::json())
        .and_then(create_user);

    let get_user_route = warp::get()
        .and(users_route.clone())
        .and(warp::any().map(move || database_copy.clone()))
        .and(warp::any().map(move || users_collection_copy.clone()))
        .and(warp::path::param::<String>())
        .and_then(get_user);

    let routes = create_user_route.or(get_user_route);

    warp::serve(routes).run(([127, 0, 0, 1], 1337)).await;

    Ok(())
}

async fn get_user(database: db::Database, collection: Collection<bson::Document>, id: String) 
-> Result<impl Reply, Rejection> {
    let result = database.find_by_id(&collection, id).await;
    
    match result {
        Ok(value) => {
            match value {
                Some(reply) => {

                    let id: &str;
                    let username: &str;
                    let password: &str;

                    let id_result = reply.get_str("id");
                    match id_result {
                        Ok(value) => {
                            id = value;
                        }
                        Err(error) => {
                            let response: String = error.to_string();
                            return Ok(warp::reply::json(&response))
                        }
                    }

                    let username_result = reply.get_str("username");
                    match username_result {
                        Ok(value) => {
                            username = value;
                        }
                        Err(error) => {
                            let response: String = error.to_string();
                            return Ok(warp::reply::json(&response))
                        }
                    }

                    let password_result = reply.get_str("password");
                    match password_result {
                        Ok(value) => {
                            password = value;
                        }
                        Err(error) => {
                            let response: String = error.to_string();
                            return Ok(warp::reply::json(&response))
                        }
                    }

                    let document = doc! {
                        "username": username,
                        "password": password,
                        "id": id
                    };

                    return Ok(warp::reply::json(&document));
                }
                None => {
                    let reply: &str = "Not found";
                    return Ok(warp::reply::json(&reply));
                }
            }
        }
        Err(error) => {
            let reply: String = error.to_string();
            Ok(warp::reply::json(&reply))
        }
    }
}

async fn create_user(database: db::Database, collection: Collection<bson::Document>, user: User) 
-> Result<impl Reply, Rejection> {
    let document = doc! {
        "username": user.username,
        "password": user.password,
        "id": Uuid::new_v4().to_string()
    };

    let result = database.insert_document(&collection, document).await;

    match result {
        Ok(_) => {
            let reply: &str = "it work";
            Ok(warp::reply::json(&reply))
        }
        Err(error) => {
            let reply: String = error.to_string();
            Ok(warp::reply::json(&reply))
        }
    }
}