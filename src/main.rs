mod db;

use serde::{Deserialize, Serialize};
use warp::{Filter, Reply, Rejection, reply::with_header};
use mongodb::{error::Error, bson, bson::{doc, oid::ObjectId}, Collection};

#[derive(Deserialize, Serialize)]
struct User {
    username: String,
    password: String
}

#[derive(Deserialize, Serialize)]
struct Note {
    content: bson::Document,
    owner_id: bson::oid::ObjectId
}

#[derive(Clone)]
struct Dependencies {
    database: db::Database,
    users_collection: Collection<bson::Document>,
    notes_collection: Collection<bson::Document>
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let database = db::Database::new().await?;
    let users_collection = database.collection("users");
    let notes_collection = database.collection("notes");

    let deps = Dependencies {
        database: database.clone(),
        users_collection: users_collection.clone(),
        notes_collection: notes_collection.clone()
    };

    let routes = setup_user_routes(deps.clone())
        .or(setup_note_routes(deps.clone()))
        .or(setup_auth_routes(deps.clone()))
        .map(|reply| {
            warp::reply::with_header(reply, "Access-Control-Allow-Origin", "*")
        });

    warp::serve(routes).run(([127, 0, 0, 1], 1337)).await;

    Ok(())
}

fn setup_note_routes(deps: Dependencies) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let notes_route = warp::path("notes");

    let create_note_route = warp::post()
        .and(notes_route.clone())
        .and(with_deps(deps.clone()))
        .and(warp::body::json())
        .and_then(create_note);

    let get_note_route = warp::get()
        .and(notes_route)
        .and(with_deps(deps.clone()))
        .and(warp::path::param::<String>())
        .and_then(get_note);

    let patch_note_route = warp::patch()
        .and(notes_route.clone())
        .and(with_deps(deps.clone()))
        .and(warp::path::param::<String>())
        .and(warp::body::json())
        .and_then(update_note);

    create_note_route.or(get_note_route).or(patch_note_route)
}

fn setup_user_routes(deps: Dependencies) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let users_route = warp::path("users");

    let create_user_route = warp::post()
        .and(users_route.clone())
        .and(with_deps(deps.clone()))
        .and(warp::body::json())
        .and_then(create_user);

    let get_user_route = warp::get()
        .and(users_route)
        .and(with_deps(deps.clone()))
        .and(warp::path::param::<String>())
        .and(warp::path::end())
        .and_then(get_user);

    let patch_user_route = warp::patch()
        .and(users_route.clone())
        .and(with_deps(deps.clone()))
        .and(warp::path::param::<String>())
        .and(warp::body::json())
        .and_then(update_user);

    let get_user_notes_route = warp::get()
        .and(users_route.clone())
        .and(with_deps(deps.clone()))
        .and(warp::path::param::<String>())
        .and(warp::path("notes"))
        .and_then(get_user_notes);

    create_user_route.or(get_user_route).or(patch_user_route).or(get_user_notes_route)
}

fn setup_auth_routes(deps: Dependencies) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let auth_route = warp::path("auth");

    let auth_route = warp::post()
        .and(auth_route.clone())
        .and(with_deps(deps.clone()))
        .and(warp::body::json())
        .and_then(get_user_id);

    auth_route
}

fn with_deps(deps: Dependencies) -> impl Filter<Extract = (Dependencies,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || deps.clone())
}

async fn get_user(deps: Dependencies, id: String) -> Result<impl Reply, Rejection> {
    let object_id: ObjectId;
    match ObjectId::parse_str(id) {
        Ok(value) => object_id = value,
        Err(_) => return Err(warp::reject::reject())
    };

    let result = deps.database.find_by_id(&deps.users_collection, object_id).await;
    
    match result {
        Ok(value) => {
            match value {
                Some(reply) => return Ok(warp::reply::json(&reply)),
                None => return Ok(warp::reply::json(&"404 not found".to_string()))
            }
        }
        Err(error) => Ok(warp::reply::json(&error.to_string()))
    }
}

async fn create_user(deps: Dependencies, user: User) -> Result<impl Reply, Rejection> {
    let document = doc! {
        "username": user.username,
        "password": user.password
    };

    let result = deps.database.insert_document(&deps.users_collection, document).await;

    match result {
        Ok(_) => Ok(warp::reply::json(&"200 OK".to_string())),
        Err(error) => Ok(warp::reply::json(&error.to_string()))
    }
}

async fn update_user(deps: Dependencies, id: String, new_content: bson::Document) -> Result<impl Reply, Rejection> {
    let object_id: ObjectId;
    match ObjectId::parse_str(id) {
        Ok(value) => object_id = value,
        Err(_) => return Err(warp::reject::reject())
    };

    let update = doc! { "$set": new_content };

    match deps.database.update_document(&deps.users_collection, object_id, update).await {
        Ok(_) => Ok(warp::reply::json(&"200 OK".to_string())),
        Err(error) => Ok(warp::reply::json(&error.to_string()))
    }
}

async fn get_user_id(deps: Dependencies, credentials: User) -> Result<impl Reply, Rejection> {    
    let filter = doc! {
        "username": credentials.username,
        "password": credentials.password
    };

    let result = deps.database.query_documents(&deps.users_collection, filter).await;

    let user_doc: bson::Document;
    match result {
        Ok(value) => user_doc = value[0].clone(),
        Err(_) => return Err(warp::reject::reject())
    };

    Ok(warp::reply::json(&user_doc))
}

async fn get_note(deps: Dependencies, id: String) -> Result<impl Reply, Rejection> {
    let object_id: ObjectId;
    match ObjectId::parse_str(id) {
        Ok(value) => object_id = value,
        Err(_) => return Err(warp::reject::reject())
    };

    let result = deps.database.find_by_id(&deps.notes_collection, object_id).await;
    
    match result {
        Ok(value) => {
            match value {
                Some(reply) => return Ok(warp::reply::json(&reply)),
                None => return Ok(warp::reply::json(&"404 not found".to_string()))
            }
        }
        Err(error) => Ok(warp::reply::json(&error.to_string()))
    }
}

async fn create_note(deps: Dependencies, note: Note) -> Result<impl Reply, Rejection> {
    let document = doc! {
        "content": note.content,
        "owner_id": note.owner_id
    };

    let result = deps.database.insert_document(&deps.notes_collection, document).await;

    match result {
        Ok(_) => Ok(warp::reply::json(&"200 OK".to_string())),
        Err(error) => Ok(warp::reply::json(&error.to_string()))
    }
}

async fn update_note(deps: Dependencies, id: String, new_content: bson::Document) -> Result<impl Reply, Rejection> {
    let object_id: ObjectId;
    match ObjectId::parse_str(id) {
        Ok(value) => object_id = value,
        Err(_) => return Err(warp::reject::reject())
    };

    let update = doc! { "$set": new_content };

    match deps.database.update_document(&deps.notes_collection, object_id, update).await {
        Ok(_) => Ok(warp::reply::json(&"200 OK".to_string())),
        Err(error) => Ok(warp::reply::json(&error.to_string()))
    }
}

async fn get_user_notes(deps: Dependencies, user_id: String) -> Result<impl Reply, Rejection> {
    let object_id: ObjectId;
    match ObjectId::parse_str(user_id) {
        Ok(value) => object_id = value,
        Err(_) => return Err(warp::reject::reject())
    };

    let filter = doc! {
        "owner_id": object_id
    };

    let result = deps.database.query_documents(&deps.notes_collection, filter).await;

    let docs: Vec<bson::Document>;
    match result {
        Ok(value) => docs = value,
        Err(error) => return Ok(warp::reply::json(&error.to_string()))
    }

    let mut json_array = bson::Array::new();
    for document in docs {
        json_array.push(bson::Bson::Document(document));
    }

    Ok(warp::reply::json(&json_array))
}
