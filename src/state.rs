use serde::{Deserialize, Serialize};

use crate::ogs::{Game, User};

#[derive(Serialize, Deserialize)]
pub struct State {
    pub logged_in_user: User,
    pub games_awaiting_move: Vec<Game>,
}
