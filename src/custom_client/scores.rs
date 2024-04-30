use rosu_v2::model::score::Score;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Scores {
    scores: Vec<Score>,
}

impl Scores {
    pub fn get(self) -> Vec<Score> {
        self.scores
    }
}
