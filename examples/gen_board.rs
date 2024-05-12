use clap::Parser;

use samegame_sfc_medium::*;

/// ゲーム内乱数生成器にパラメータを与えて盤面を生成する。
#[derive(Debug, Parser)]
struct Cli {
    #[arg(value_parser = parse_int::parse::<u16>)]
    state: u16,

    #[arg(value_parser = parse_int::parse::<u8>)]
    counter: u8,

    #[arg(value_parser = parse_int::parse::<usize>)]
    inc_timing: usize,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let (board, ok) = GameRng::new(cli.state).gen_board(cli.counter, cli.inc_timing);
    if !ok {
        eprintln!("WARN: 再生成判定に引っ掛かる");
    }

    print!("{board}");

    Ok(())
}
