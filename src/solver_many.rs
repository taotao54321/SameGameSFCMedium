use std::num::NonZeroU64;

use crate::action::ActionHistory;
use crate::board::Board;
use crate::cmp::chmax;
use crate::position::Position;
use crate::rng::GameRng;
use crate::score::{calc_score_erase, Score, SCORE_PERFECT};

/// ある盤面集合内で最大スコアを求めた結果の解。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SolutionMany {
    rng_state: u16,
    rng_counter: u8,
    rng_inc_timing: usize,
    score: Score,
    solution: ActionHistory,
}

impl SolutionMany {
    pub fn rng_state(&self) -> u16 {
        self.rng_state
    }

    pub fn rng_counter(&self) -> u8 {
        self.rng_counter
    }

    pub fn rng_inc_timing(&self) -> usize {
        self.rng_inc_timing
    }

    pub fn score(&self) -> Score {
        self.score
    }

    pub fn solution(&self) -> &ActionHistory {
        &self.solution
    }
}

/// 与えられた盤面集合内で最大スコアを求める。
pub fn solve_problems_many<I>(rng_params: I, best_score_ini: Score) -> Option<SolutionMany>
where
    I: IntoIterator<Item = (u16, u8, usize)>,
{
    Solver::new(best_score_ini).solve(rng_params)
}

fn gen_board(state: u16, counter: u8, inc_timing: usize) -> (Board, bool) {
    GameRng::new(state).gen_board(counter, inc_timing)
}

#[derive(Debug)]
struct Solver {
    best_score: Score,
    best_ans: Option<SolutionMany>,
    dp: DpTable,
}

impl Solver {
    fn new(best_score_ini: Score) -> Self {
        Self {
            best_score: best_score_ini,
            best_ans: None,
            dp: DpTable::new(),
        }
    }

    fn solve<I>(mut self, rng_params: I) -> Option<SolutionMany>
    where
        I: IntoIterator<Item = (u16, u8, usize)>,
    {
        for (rng_state, rng_counter, rng_inc_timing) in rng_params {
            // 再生成判定に引っ掛かる場合はスキップ。
            // なお gen_board() で盤面生成するので、初期盤面が空のケースは考えなくてよい。
            let (board, true) = gen_board(rng_state, rng_counter, rng_inc_timing) else {
                eprintln!("Regen: state=0x{rng_state:04X} counter=0x{rng_counter:02X} inc_timing={rng_inc_timing}");
                continue;
            };
            let pos = Position::new(board);

            // 最大スコアを超えられない局面は枝刈り。
            if is_prunable(self.best_score, &pos) {
                eprintln!("Pruned: state=0x{rng_state:04X} counter=0x{rng_counter:02X} inc_timing={rng_inc_timing}");
                continue;
            }

            eprintln!("Search: state=0x{rng_state:04X} counter=0x{rng_counter:02X} inc_timing={rng_inc_timing}");

            let (score, solution) = self.solve_one(&pos);
            if chmax!(self.best_score, score) {
                eprintln!("Found {score}: state=0x{rng_state:04X} counter=0x{rng_counter:02X} inc_timing={rng_inc_timing}");
                self.best_ans.replace(SolutionMany {
                    rng_state,
                    rng_counter,
                    rng_inc_timing,
                    score,
                    solution,
                });
            }

            self.dp.increment_time();
        }

        self.best_ans
    }

    fn solve_one(&mut self, pos_root: &Position) -> (Score, ActionHistory) {
        let score = self.dfs(pos_root);
        eprintln!("DP entry count: {}", self.dp.entry_count());

        // 経路復元。
        let mut solution = ActionHistory::new();
        let mut pos = pos_root.clone();
        loop {
            let best_action = pos.actions().max_by_key(|action| {
                let pos_child = pos.do_action(action);
                let gain_action = calc_score_erase(action.square_count());
                // 空の盤面は DP テーブルに載らないので例外処理が必要。
                // それ以外の盤面は DP テーブルに載っているはず。
                let gain_child = if pos_child.board().is_empty() {
                    SCORE_PERFECT
                } else {
                    let DpTableProbe::Found(score) = self.dp.probe(pos_child.key()) else {
                        eprintln!("この盤面の DP エントリが見つからない!?");
                        eprint!("{}", pos_child.board());
                        unreachable!();
                    };
                    score
                };
                gain_action + gain_child
            });
            let Some(best_action) = best_action else {
                break;
            };
            solution.push(best_action.least_square());
            pos = pos.do_action(&best_action);
        }

        (score, solution)
    }

    /// メモ化再帰。戻り値は `pos` から追加で獲得できる最大スコア。
    fn dfs(&mut self, pos: &Position) -> Score {
        // 空の盤面に対する DP エントリが作られないよう、先にパーフェクト判定する。
        // 他の終了局面については仮作成するエントリの gain_max が 0 なのでそのままでよい。
        if pos.board().is_empty() {
            return SCORE_PERFECT;
        }

        let key = pos.key();

        match self.dp.probe(key) {
            DpTableProbe::Found(gain_max) => gain_max,
            DpTableProbe::Created(dp_idx) => {
                let mut gain_max = 0;
                for action in pos.actions() {
                    let pos_child = pos.do_action(&action);
                    let gain_action = calc_score_erase(action.square_count());
                    let gain_child = self.dfs(&pos_child);
                    chmax!(gain_max, gain_action + gain_child);
                }

                // 終了局面ならば単に 0 を返す。
                // DP テーブルに仮作成したエントリの gain_max は 0 なのでそのままでよい。
                // 終了局面であることと gain_max が 0 であることは同値。
                if gain_max == 0 {
                    return 0;
                }

                self.dp.set_gain_max(dp_idx, gain_max);
                gain_max
            }
        }
    }
}

/// 既知の最大スコアが `best_score` であるときに初期局面 `pos` を枝刈りできるかどうかを返す。
fn is_prunable(best_score: Score, pos: &Position) -> bool {
    // 浅い探索を行ってスコア上界を見積もり、それが best_score 以下ならば枝刈り可能とする。
    // もっと賢い方法がありそうだが、メモ化再帰とスコア上界による枝刈りを組み合わせるのが難しかったので...。

    for depth in 0..=3 {
        if is_prunable_dfs(pos, depth) <= best_score {
            return true;
        }
    }

    false
}

/// 戻り値は `pos` から追加で獲得しうるスコアの上界。
fn is_prunable_dfs(pos: &Position, depth_remain: u32) -> Score {
    if depth_remain == 0 {
        return pos.score_upper_bound();
    }

    let mut score_ub_max = 0;
    for action in pos.actions() {
        let pos_child = pos.do_action(&action);
        let gain_action = calc_score_erase(action.square_count());
        let score_ub_child = is_prunable_dfs(&pos_child, depth_remain - 1);
        chmax!(score_ub_max, gain_action + score_ub_child);
    }

    // 終了局面ならばパーフェクト判定して値を返す。
    // 終了局面であることと score_ub_max が 0 であることは同値。
    if score_ub_max == 0 {
        return if pos.board().is_empty() {
            SCORE_PERFECT
        } else {
            0
        };
    }

    score_ub_max
}

const DP_TABLE_CAP_BITS: u32 = 30;
const DP_TABLE_CAP: usize = 1 << DP_TABLE_CAP_BITS;
const _: () = assert!(DP_TABLE_CAP.is_power_of_two());

const KEY_HI_BITS: u32 = 64 - DP_TABLE_CAP_BITS;
const KEY_HI_SHIFT: u32 = 64 - KEY_HI_BITS;
const KEY_HI_MASK: u64 = ((1 << KEY_HI_BITS) - 1) << KEY_HI_SHIFT;

fn calc_key_hi(key: u64) -> u64 {
    key >> KEY_HI_SHIFT
}

/// DP テーブルのエントリ。
///
/// * bit 0-15: 世代 (DP テーブルを毎回再初期化せずに済ませるための機構)。
/// * bit16-28: 1 + (この局面から追加で獲得できる最大スコア)。
/// * bit29   : (未使用)
/// * bit30-63: この局面のハッシュ値の上位部分。
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct DpEntry(NonZeroU64);

const _: () = assert!(std::mem::size_of::<Option<DpEntry>>() == 8);

impl DpEntry {
    const TIME_BITS: u32 = 16;
    const TIME_MASK: u64 = (1 << Self::TIME_BITS) - 1;

    const GAIN_MAX_BITS: u32 = 13;
    const GAIN_MAX_SHIFT: u32 = 16;
    const GAIN_MAX_MASK: u64 = ((1 << Self::GAIN_MAX_BITS) - 1) << Self::GAIN_MAX_SHIFT;

    fn new(time: u16, key: u64, gain_max: Score) -> Self {
        let value_time = u64::from(time);
        let value_gain_max = u64::from(1 + gain_max) << Self::GAIN_MAX_SHIFT;
        let value_key = key & KEY_HI_MASK;
        let value = value_time | value_gain_max | value_key;

        Self(unsafe { NonZeroU64::new_unchecked(value) })
    }

    fn time(self) -> u16 {
        (self.0.get() & Self::TIME_MASK) as u16
    }

    fn gain_max(self) -> Score {
        (((self.0.get() & Self::GAIN_MAX_MASK) >> Self::GAIN_MAX_SHIFT) - 1) as Score
    }

    fn set_gain_max(&mut self, gain_max: Score) {
        let value_gain_max = u64::from(1 + gain_max) << Self::GAIN_MAX_SHIFT;
        let value = (self.0.get() & !Self::GAIN_MAX_MASK) | value_gain_max;

        self.0 = unsafe { NonZeroU64::new_unchecked(value) };
    }

    fn key_hi(self) -> u64 {
        self.0.get() >> KEY_HI_SHIFT
    }
}

/// DP テーブル。
///
/// 世代情報を用いることで、配列を再初期化することなく 0x10000 個の問題を続けて解ける。
#[derive(Debug)]
struct DpTable {
    time: u16,
    entry_count: usize,
    array: Box<[Option<DpEntry>; DP_TABLE_CAP]>,
}

impl DpTable {
    const INDEX_MASK: usize = DP_TABLE_CAP - 1;

    fn new() -> Self {
        Self {
            time: 0,
            entry_count: 0,
            array: vec![None; DP_TABLE_CAP].try_into().unwrap(),
        }
    }

    /// 現在の世代を返す。
    #[allow(dead_code)]
    fn time(&self) -> u16 {
        self.time
    }

    /// 現在の世代におけるエントリ数を返す。
    fn entry_count(&self) -> usize {
        self.entry_count
    }

    /// 世代を更新する。
    ///
    /// 世代がオーバーフローする場合のみテーブル全体が再初期化される。
    fn increment_time(&mut self) {
        let overflow;
        (self.time, overflow) = self.time.overflowing_add(1);

        self.entry_count = 0;

        if overflow {
            self.array.fill(None);
        }
    }

    /// 現在の世代においてハッシュ値 `key` に対応するエントリを探す。
    ///
    /// エントリが既に存在する場合、その値 (gain_max) を返す。
    /// エントリがまだ存在しない場合、仮の値でエントリを作成し、そのインデックスを返す。
    fn probe(&mut self, key: u64) -> DpTableProbe {
        // key に対応する局面が終了局面の場合、盤面が空でないなら仮作成したエントリはそのままにできる。
        // (gain_max を 0 として仮作成するので)
        // しかし、盤面が空の場合は仮作成してしまうとエントリの値が正しくなくなる。
        //
        // これを安直に解決するならハッシュ値 0 に対して SCORE_PERFECT を返すようにすればよいが、
        // 偶然ハッシュ値が 0 の空でない盤面が生じてしまうとほぼ確実に解がおかしくなる。
        //
        // というわけで、一応 Solver 側で空の盤面に対する例外処理を行い、
        // 空の盤面は DP テーブルに載らないようにしておく。

        // linear probing
        let mut idx = key as usize & Self::INDEX_MASK;
        loop {
            let entry = unsafe { self.array.get_unchecked_mut(idx) };

            macro_rules! return_created {
                () => {{
                    self.entry_count += 1;
                    /*
                    if self.entry_count.is_power_of_two() {
                        eprintln!("DP entry count: {}", self.entry_count);
                    }
                    */
                    entry.replace(DpEntry::new(self.time, key, 0));
                    return DpTableProbe::Created(idx);
                }};
            }

            match entry {
                None => return_created!(),
                Some(entry) if entry.time() != self.time => return_created!(),
                Some(entry) if entry.key_hi() == calc_key_hi(key) => {
                    return DpTableProbe::Found(entry.gain_max());
                }
                _ => idx = (idx + 1) & Self::INDEX_MASK,
            }
        }
    }

    /// `probe()` で仮作成したエントリの値を `gain_max` に設定する。
    fn set_gain_max(&mut self, idx: usize, gain_max: Score) {
        let entry = unsafe { self.array.get_unchecked_mut(idx) };
        let entry = unsafe { entry.as_mut().unwrap_unchecked() };

        entry.set_gain_max(gain_max);
    }
}

#[derive(Debug)]
enum DpTableProbe {
    Found(Score),
    Created(usize),
}
