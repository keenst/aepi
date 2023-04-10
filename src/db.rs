use mongodb::{
    bson::{Document, doc, self, document::ValueAccessError},
    error::Error,
    options::{ClientOptions, FindOptions, FindOneOptions},
    Client,
};
use futures::TryStreamExt;

#[derive(Clone)]
pub struct Database {
    client: Client
}

impl Database {
    pub async fn new() -> Result<Self, Error> {
        let client_options = ClientOptions::parse(
            "mongodb+srv://admin:yoFQAyeaTZNAFFam@aepi.umea0mi.mongodb.net/?retryWrites=true&w=majority")
            .await?;
        let client = Client::with_options(client_options)?;
        Ok(Database { client })
    }

    pub fn collection(&self, name: &str) -> mongodb::Collection<Document> {
        let db = self.client.database("public");
        db.collection(name)
    }

    pub async fn insert_document(&self, collection: &mongodb::Collection<Document>, document: Document) 
    -> Result<(), Error> {
        collection.insert_one(document, None).await?;
        Ok(())
    }

    pub async fn all_documents(&self, collection: &mongodb::Collection<Document>) -> Result<Vec<Document>, Error> {
        let find_options = FindOptions::builder().build();
        let mut cursor = collection.find(None, find_options).await?;
        let mut documents = Vec::new();
        while let Some(result) = cursor.try_next().await? {
            documents.push(result);
        }
        Ok(documents)
    }

    pub async fn find_by_id(&self, collection: &mongodb::Collection<Document>, id: String) 
    -> Result<Option<bson::Document>, Error> {
        let query = doc! {
            "id": id
        };
        let options = FindOneOptions::builder().build();
        collection.find_one(query, options).await
    }
}