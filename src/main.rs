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
struct Note {
    content: bson::Document,
    owner_id: Uuid,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let database = db::Database::new().await?;

    // users routes
    let users_collection = database.collection("users");
    let users_collection_copy = users_collection.clone();
    let users_route = warp::path("users");

    let database_copy = database.clone();

    let create_user_route = warp::post()
        .and(users_route.clone())
        .and(warp::any().map(move || database.clone()))
        .and(warp::any().map(move || users_collection.clone()))
        .and(warp::body::json())
        .and_then(create_user);

    let database_copy_2 = database_copy.clone();

    let get_user_route = warp::get()
        .and(users_route.clone())
        .and(warp::any().map(move || database_copy.clone()))
        .and(warp::any().map(move || users_collection_copy.clone()))
        .and(warp::path::param::<String>())
        .and_then(get_user);

    // notes routes
    let notes_collection = database_copy_2.collection("notes");
    let notes_collection_copy = notes_collection.clone();
    let notes_route = warp::path("notes");

    let database_copy_3 = database_copy_2.clone();

    let create_note_route = warp::post()
        .and(notes_route.clone())
        .and(warp::any().map(move || database_copy_2.clone()))
        .and(warp::any().map(move || notes_collection.clone()))
        .and(warp::body::json())
        .and_then(create_note);
        
    let get_note_route = warp::get()
        .and(notes_route.clone())
        .and(warp::any().map(move || database_copy_3.clone()))
        .and(warp::any().map(move || notes_collection_copy.clone()))
        .and(warp::path::param::<String>())
        .and_then(get_note);

    let routes = create_user_route
        .or(get_user_route)
        .or(create_note_route)
        .or(get_note_route);

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
            Ok(warp::reply::json(&"200 OK".to_string()))
        }
        Err(error) => {
            Ok(warp::reply::json(&error.to_string()))
        }
    }
}

async fn get_note(database: db::Database, collection: Collection<bson::Document>, id: String) 
-> Result<impl Reply, Rejection> {
    let result = database.find_by_id(&collection, id).await;
    
    match result {
        Ok(value) => {
            match value {
                Some(reply) => {

                    let content: &bson::Document;
                    let owner_id: &str;
                    let id: &str;

                    match reply.get_document("content") {
                        Ok(value) => {
                            content = value;
                        }
                        Err(error) => {
                            let response: String = error.to_string();
                            return Ok(warp::reply::json(&response))
                        }
                    }

                    match reply.get_str("owner_id") {
                        Ok(value) => {
                            owner_id = value;
                        }
                        Err(error) => {
                            let response: String = error.to_string();
                            return Ok(warp::reply::json(&response))
                        }
                    }

                    match reply.get_str("id") {
                        Ok(value) => {
                            id = value;
                        }
                        Err(error) => {
                            let response: String = error.to_string();
                            return Ok(warp::reply::json(&response))
                        }
                    }

                    let document = doc! {
                        "content": content,
                        "owner_id": owner_id,
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

async fn create_note(database: db::Database, collection: Collection<bson::Document>, note: Note)
-> Result<impl Reply, Rejection> {
    let document = doc! {
        "content": note.content,
        "owner_id": note.owner_id.to_string(),
        "id": Uuid::new_v4().to_string()
    };

    let result = database.insert_document(&collection, document).await;

    match result {
        Ok(_) => {
            Ok(warp::reply::json(&"200 OK".to_string()))
        }
        Err(error) => {
            Ok(warp::reply::json(&error.to_string()))
        }
    }
}