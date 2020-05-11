use postgres::{Client, Error, NoTls};

pub fn get_db_client() -> Result<Client, Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let client = Client::connect(&database_url, NoTls)?;
    return Ok(client);
}
