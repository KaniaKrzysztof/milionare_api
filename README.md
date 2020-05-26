# API do gry "Milionerzy"

Część serwerowa aplikacji "Milionerzy" spełnia zadania:

* Pobieranie zestawu losowych pytań do pojedynczej rozgrywki
* Wypełnianie bazy pytaniami pobranymi z zewnętrznego źródła

API dostępne jest pod adresem [API](https://whispering-dusk-90520.herokuapp.com/)

# Technologie

W celu odseparowania logiki związanej z samą mechaniką gry zdecydowaliśmy, aby część serwerowa została zaimplementowana w postaci RESTowego API. Rozwiązanie całkowicie uniezależnia częśc serwerową od warstwy prezentacyjnej, która może zostać wykonana w dowolnej technologii oraz platformie. Jednocześnie pozostaje ona najbardziej elastyczna i skalowalna, pozwalając na bezproblemową rozbudowę w przyszłości.

API zostało napisane w języku Rust, nowatorskiej technologii łączacej wydajność języków niskopoziomowych jak C oraz łatwość pisania języków wysokopoziomowych jak Python. Dodatkowo dzięki systemowi pożyczania oraz statycznemu typowaniu, Rust gwarantuje bezpieczeństwo pamięci aplikacji w momencie kompilacji, pozwalając na szybkie prototypowanie bez obaw o wycieki pamięci, czy błędy typu NullPointerException.

Jako bazę danych wybrałem PostreSQL, ponieważ jest zdecydowanie bardziej rozbudowany od MySql oraz pozwala na prostą integracją z hostingiem Heroku.

# Struktura aplikacji

1. [Zależności](#Zależności)
2. [Moduły aplikacji](#Moduły-aplikacji)
    1. [main.rs](#mainrs)
    2. [db.rs](#dbrs)
    3. [question.rs](#questionrs)
    4. [jparse.rs](#jparsers)
    5. [download_and_upload.rs](#download_and_uploadrs)
    6. [lib.rs](#librs)

# Zależności

``` rust
[package]
name = "milionares_api"
version = "0.1.0"
authors = ["Krzysztof Kania <kontakt@krzysztofkania.com>"]
edition = "2018"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
rocket = "0.4.4"  // framework web servera API
dotenv = "0.15.0" // pobieranie danych ze zmiennych środowiskowych
postgres = "0.17.3" // klient bazy danych postgreSQL
serde = { version = "1.0.110", features = ["derive"] } // serializer obiektów do JSON
reqwest = "0.9.24" // klient http
serde_json = "1.0.53" // deserializer JSON


[dependencies.rocket_contrib]
version = "*"
default-features = false
features = ["json"]
```

# Moduły aplikacji

## main.rs

### Deklaracja makr oraz używanych bibliotek

``` rust
#![feature(proc_macro_hygiene, decl_macro)]
#[macro_use]
extern crate rocket;
extern crate reqwest;
extern crate serde;

use dotenv::dotenv;
use milionares_api::db;
use milionares_api::download_and_upload;
use milionares_api::question;
use rocket::http::RawStr;
use rocket_contrib::json::Json;
use std::env;
```

### Zdefiniowanie endpointów API

Głównym endpointem API jest /api. Zwraca on w formacie JSON tablicę 12 pytań o 3 różnych poziomach trudności:

``` javascript
{
  "id": 47,
  "question": "Terry Gilliam was an animator that worked with which British comedy group?",
  "difficulty": "easy",
  "incorrect_answers":
 [
    "The Goodies&lrm;",
    "The League of Gentlemen&lrm;",
    "The Penny Dreadfuls"
 ],
 "correct_answer": "Monty Python"
}
```

Funkcja ```api()``` otwiera nowe połączenie z bazą danych, a następnie przekazuje je do metody ```get_questions()``` pobierającej pytania i zwracająca je w postaci wektora z pytaniami. Dane zostają zserializowane dzięki funkcji ```Json()```.

``` rust
#[get("/api")]
fn api() -> Json<Vec<question::Question>> {
    let mut db_client = db::get_db_client().unwrap();

    let q = question::get_questions(&mut db_client).unwrap();

    Json(q)
}
```

Drugi endpoint ```/upload?password=[hasło]``` wywołuje procedurę pobrania pytań z zewnętrznego źródła i zapisania ich do bazy danych. Jest to możliwe tylko jeżeli pod parametrem password zostanie podane hasło, które jest zapisane na hostingo w zmiennej środowiskowej.

``` rust
#[get("/upload?<password>")]
fn upload(password: &RawStr) -> String {
    let upload_pass = env::var("UPLOAD_PASS").expect("upload password env error");
    if upload_pass == password.to_string() {
        let _ = download_and_upload::fill_db_with_questions();
        return "upload sucess".to_string();
    } else {
        return "upload failed".to_string();
    }
}
```

Ostatni endpoint to tzw. easter egg, można dzięki niemu sprawdzić czy usługa działa.

``` rust
#[get("/")]
fn index() -> String {
    "Hello millionare!".to_string()
}
```

### Główny kod

W głównej funkcji programu następuje jedynie załadowanie zmiennych środowiskowych oraz uruchomienie serwera i podłączenie endpointów.

``` rust
fn main() {
    dotenv().ok();
    rocket::ignite()
        .mount("/", routes![api])
        .mount("/", routes![index])
        .mount("/", routes![upload])
        .launch();
}

```

## db.rs

Moduł db.rs odpowiada za utworzenie połączenia z bazą danych.

### ```get_db_client()```

Funkcja ```get_db_client()``` pobiera ze zmiennej środowiskowej adres oraz dane logowania do bazy danych ze zmiennej środowiskowej. Następnie za pomocą biblioteki postgres tworzy Clienta umożliwiającego wykonywanie zapytań.

``` rust
use postgres::{Client, Error, NoTls};

pub fn get_db_client() -> Result<Client, Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let client = Client::connect(&database_url, NoTls)?;
    return Ok(client);
}
```

## question.rs

Moduł question.rs to model danych zawierający informacje o pytaniu oraz wiedzę domenową związane z modelem jak np. poziomy trudności pytań oraz metodę pobierania danych z bazy.

### Model danych

Podstawą modułu jest struktura  ```Question``` reprezentująca model danych aplikacji. Dodatkowo dzięki dyrektywie ```#[derive(Deserialize, Serialize, Debug)]``` z biblioteki ```serde```, model ten jest automatycznie de/serializowany.

``` rust
#[derive(Deserialize, Serialize, Debug)]
pub struct Question {
    pub id: i32,
    pub question: String,
    pub difficulty: String,
    pub incorrect_answers: Vec<String>,
    pub correct_answer: String,
}
```

Dodatkowo do ```Question``` dołączony jest typ enumerowany ```Difficulty``` oraz statyczna tablica oparta na tym typie powalająca na iterację po poziomach trudności i wykorzystywanie zmapowanych do nich danych.

``` rust
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    pub fn as_str(&self) -> &str {
        match self {
            &Difficulty::Easy => "easy",
            &Difficulty::Medium => "medium",
            &Difficulty::Hard => "hard",
        }
    }
}

pub const DIFFICULTIES_ARRAY: [Difficulty; 3] =
    [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard];
```

### Pobieranie danych

Metoda ```get_questions()``` odpowiada za pobieranie pytań z bazy. Przyjmuje jako argument clienta bazy danych pobieranego z db.rs przez ```get_db_client()```. Metoda ta iteruje po wcześniej opisanej tablicy wykonując sparametryzowane zapytanie do tablicy ```question``` i łaczy dane w jeden wektor.

``` rust
pub fn get_questions(db_client: &mut postgres::Client) -> Result<Vec<Question>, postgres::Error> {
    let mut downloaded_question: Vec<Question> = Vec::new();

    for question_difficulty in DIFFICULTIES_ARRAY.iter() {
        let amount: i64 = 4;
        for row in db_client.query(
            "SELECT id, question, incorrect_answers, correct_answer, difficulty FROM question WHERE question.difficulty = $1 ORDER BY random() LIMIT $2 ",
            &[&question_difficulty.as_str(), &amount]
            )? {
            let question = Question {
                id: row.get(0),
                question: row.get(1),
                incorrect_answers: row.get(2),
                correct_answer: row.get(3),
                difficulty: row.get(4),
            };
            downloaded_question.push(question);
        }
    }

    Ok(downloaded_question)
}
```

## jparse.rs

Moduł ```jparse.rs``` odpowiada za parsowanie i deserializację danych pobranych w formacie JSON z zewnętrznej bazy pytań.

### Model danych

W tym module snajdują się 2 deserializowalne struktury:

* ```Response``` - struktura "wrappera" odpowiedzi zawierjącej właściwe dane
* ```QuestionFromJson``` - struktura do której mapujemy dane pytań

``` rust
#[derive(Deserialize, Debug)]
pub struct QuestionFromJson {
    pub question: String,
    pub difficulty: String,
    pub incorrect_answers: Vec<String>,
    pub correct_answer: String,
}

#[derive(Deserialize)]
struct Response {
    response_code: u32,
    results: Vec<QuestionFromJson>,
}
```

### ```parse_questions_from_jsonstr()``` 

Metoda przyjmuje ciąg znaków zawierający dane w formacie JSON, a następnie za pomocją biblioteki ```serde_json``` deserializuje je do struktur danych.

``` rust
pub fn parse_questions_from_jsonstr(data: &str) -> Result<Vec<QuestionFromJson>> {
    let downloaded_data: Response = serde_json::from_str(data)?;
    let questions_data = downloaded_data.results;
    Ok(questions_data)
}
```

## download_and_upload.rs

Moduł ten odpowiada za pobranie danych w formacie JSON z zewnętrznej bazy poprzez klienta http według zadanych parametrów, a następnie wprowadzenie ich do naszej bazy danych.

### ```upload_question_to_db()```

Metoda przyjmuje strukturę ```QuestionFromJson``` z modułu jparse.rs oraz klienta bazy danych, a następnie parametryzuje zapytanie wstawiające pobrane dane do bazy.

``` rust
fn upload_question_to_db(
    question_data: &crate::jparse::QuestionFromJson,
    db_client: &mut postgres::Client,
) -> Result<(), postgres::Error> {
    db_client.execute(
        "
        INSERT INTO question (
            question, incorrect_answers, correct_answer, difficulty)
            VALUES ($1, $2, $3, $4);
    ",
        &[
            &question_data.question,
            &question_data.incorrect_answers,
            &question_data.correct_answer,
            &question_data.difficulty,
        ],
    )?;
    Ok(())
}
```

### ```fill_db_with_questions()```

Metoda pobiera 50 pytań za pomocą klienta http z zewnętrznej bazy pytania w formacie JSON. Następnie deserializuje je za pomocą modułu ```jparse.rs``` i wstawia dane do bazy za pomocą ```upload_question_to_db()```.

``` rust
pub fn fill_db_with_questions() -> Result<(), Error> {
    let amount = 50;

    for difficulty in crate::question::DIFFICULTIES_ARRAY.iter() {
        let request_url = format!(
            "https://opentdb.com/api.php?amount={amount}&difficulty={difficulty}&type=multiple",
            difficulty = &difficulty.as_str(),
            amount = &amount,
        );

        let response = reqwest::get(&request_url)?.text()?;

        let questions_data = jparse::parse_questions_from_jsonstr(&response).unwrap();
        let mut db_client = db::get_db_client().unwrap();
        for single_question_data in questions_data.iter() {
            upload_question_to_db(single_question_data, &mut db_client)
                .expect("Question upload error");
        }
    }
    Ok(())
}
```

## lib.rs

Moduł odpowiadający za podłączenie modułów do programu.

``` rust
pub mod db;
pub mod download_and_upload;
pub mod jparse;
pub mod question;
```
