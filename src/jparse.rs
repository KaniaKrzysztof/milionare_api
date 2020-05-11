use serde::Deserialize;
use serde_json::Result;

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

pub fn parse_questions_from_jsonstr(data: &str) -> Result<Vec<QuestionFromJson>> {
    let downloaded_data: Response = serde_json::from_str(data)?;
    let questions_data = downloaded_data.results;
    Ok(questions_data)
}
