//! スコア関連。

use crate::hint::assert_unchecked;
use crate::square::Square;

/// スコア型。
///
/// 理論上の値域は `0..=6741` (最大値は 80 個全消し時)。
/// つまり 13bit に収まる。
pub type Score = u32;

/// 「ふつう」サイズの盤面における理論上の最大スコア。
/// ゲーム中では実現不可 (同種駒の個数に制限があるため)。
pub const SCORE_MAX: Score = calc_score_erase(Square::NUM as u32) + SCORE_PERFECT;

/// パーフェクト達成時に得られるボーナススコア。
pub const SCORE_PERFECT: Score = 500;

/// n 個の駒を消す着手による獲得スコアを返す。
///
/// `n >= 2` でなければならない。
pub const fn calc_score_erase(n: u32) -> Score {
    unsafe { assert_unchecked!(n >= 2) }

    (n - 1).pow(2)
}
