use serde::{Deserialize, Serialize};

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

#[derive(Deserialize, Serialize, Debug)]
pub struct Question {
    pub id: i32,
    pub question: String,
    pub difficulty: String,
    pub incorrect_answers: Vec<String>,
    pub correct_answer: String,
}

impl Question {}

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
