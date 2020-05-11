use crate::db;
use crate::jparse;
use reqwest;
use reqwest::Error;

pub fn fill_db_with_questions() -> Result<(), Error> {
    let amount = 50;

    for difficulty in crate::question::DIFFICULTIES_ARRAY.iter() {
        let request_url = format!(
            "https://opentdb.com/api.php?amount={amount}&difficulty={difficulty}&type=multiple",
            difficulty = &difficulty.as_str(),
            amount = &amount,
        );

        println!("{}", request_url);
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

fn upload_question_to_db(
    question_data: &crate::jparse::QuestionFromJson,
    db_client: &mut postgres::Client,
) -> Result<(), postgres::Error> {
    let t = db_client.execute(
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

    dbg!(t);

    Ok(())
}
