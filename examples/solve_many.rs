use clap::Parser;

use samegame_sfc_medium::*;

/// 与えられた乱数パラメータ集合内で最大スコアを求める。
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

    #[arg(long, default_value_t = 0, value_parser = parse_int::parse::<Score>)]
    best_score_ini: Score,
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

    let rng_params = itertools::iproduct!(states, counters, inc_timings);

    if let Some(ans) = solve_problems_many(rng_params, cli.best_score_ini) {
        println!(
            "{}\t0x{:04X}\t0x{:02X}\t{}\t{}",
            ans.score(),
            ans.rng_state(),
            ans.rng_counter(),
            ans.rng_inc_timing(),
            ans.solution()
        );
    } else {
        eprintln!("NO SOLUTION");
    }

    Ok(())
}
