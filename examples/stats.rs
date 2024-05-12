use clap::Parser;
use itertools::{Itertools as _, MinMaxResult};

use samegame_sfc_medium::*;

/// ゲーム内に現れうる面の集合について統計情報を得る。
#[derive(Debug, Parser)]
struct Cli {
    #[arg(long, default_value_t = 0, value_parser = parse_int::parse::<u16>)]
    state_min: u16,

    #[arg(long, default_value_t = u16::MAX, value_parser = parse_int::parse::<u16>)]
    state_max: u16,

    #[arg(long, default_value_t = 0, value_parser = parse_int::parse::<u8>)]
    counter_min: u8,

    #[arg(long, default_value_t = u8::MAX, value_parser = parse_int::parse::<u8>)]
    counter_max: u8,

    #[arg(long, default_value_t = 39)]
    inc_timing_min: usize,

    #[arg(long, default_value_t = 40)]
    inc_timing_max: usize,
}

fn make_range<T: PartialOrd>(min: T, max: T) -> std::ops::RangeInclusive<T> {
    assert!(min <= max);
    min..=max
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let states = make_range(cli.state_min, cli.state_max);
    let counters = make_range(cli.counter_min, cli.counter_max);
    let inc_timings = make_range(cli.inc_timing_min, cli.inc_timing_max);

    let stats = get_stats(states, counters, inc_timings);

    report(&stats);

    Ok(())
}

fn report(stats: &Stats) {
    println!(
        "piece count: {} ..= {}",
        stats.piece_count_min, stats.piece_count_max
    );
    println!(
        "score upper bound: {} ..= {}",
        stats.score_ub_min, stats.score_ub_max
    );
}

#[derive(Debug)]
struct Stats {
    /// 同種駒の個数の最小値。
    piece_count_min: u32,

    /// 同種駒の個数の最大値。
    piece_count_max: u32,

    /// スコア上界の最小値。
    score_ub_min: Score,

    /// スコア上界の最大値。
    score_ub_max: Score,
}

fn get_stats<I, J, K>(states: I, counters: J, inc_timings: K) -> Stats
where
    I: IntoIterator<Item = u16>,
    I::IntoIter: Clone,
    J: IntoIterator<Item = u8>,
    J::IntoIter: Clone,
    K: IntoIterator<Item = usize>,
    K::IntoIter: Clone,
{
    let mut piece_count_min = u32::MAX;
    let mut piece_count_max = 0;
    let mut score_ub_min = Score::MAX;
    let mut score_ub_max = 0;

    for (state, counter, inc_timing) in itertools::iproduct!(states, counters, inc_timings) {
        let (board, board_is_ok) = gen_board(state, counter, inc_timing);
        if !board_is_ok {
            eprintln!(
                "regenerate: state=0x{state:04X} counter=0x{counter:02X} inc_timing={inc_timing}"
            );
            continue;
        };

        let pos = Position::new(board);

        {
            let MinMaxResult::MinMax(min, max) = Piece::all()
                .map(|piece| u32::from(pos.piece_count(piece)))
                .minmax()
            else {
                unreachable!();
            };
            chmin!(piece_count_min, min);
            chmax!(piece_count_max, max);
            if max == (Square::NUM / 2 - 1) as u32 {
                eprintln!(
                    "piece39: state=0x{state:04X} counter=0x{counter:02X} inc_timing={inc_timing}"
                );
            }
        }

        let score_ub = pos.score_upper_bound();
        chmin!(score_ub_min, score_ub);
        chmax!(score_ub_max, score_ub);
    }

    Stats {
        piece_count_min,
        piece_count_max,
        score_ub_min,
        score_ub_max,
    }
}

fn gen_board(state: u16, counter: u8, inc_timing: usize) -> (Board, bool) {
    GameRng::new(state).gen_board(counter, inc_timing)
}
