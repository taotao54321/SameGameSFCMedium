use anyhow::Context as _;

use crate::action::{Action, ActionHistory};
use crate::board::Board;
use crate::position::Position;
use crate::score::{calc_score_erase, Score, SCORE_PERFECT};

/// 解の verify を行い、そのスコアを返す。
pub fn verify_solution(board: &Board, solution: &ActionHistory) -> anyhow::Result<Score> {
    let mut pos = Position::new(board.clone());
    let mut score = 0;

    for (i, &sq) in solution.iter().enumerate() {
        let action = Action::from_board_square(pos.board(), sq)
            .with_context(|| format!("着手[{i}] {sq} が違法"))?;
        score += calc_score_erase(action.square_count());
        pos = pos.do_action(&action);
    }

    if pos.board().is_empty() {
        score += SCORE_PERFECT;
    }

    Ok(score)
}
