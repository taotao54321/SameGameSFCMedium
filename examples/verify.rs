use std::path::PathBuf;

use anyhow::{ensure, Context as _};
use clap::Parser;

use samegame_sfc_medium::*;

/// 解の verify を行う。
#[derive(Debug, Parser)]
struct Cli {
    path_board: PathBuf,

    /// (スコア, 解) をタブで区切った 1 行のみを含むテキストファイル。
    path_solution: PathBuf,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let board = std::fs::read_to_string(&cli.path_board)
        .with_context(|| format!("問題ファイル '{}' を読めない", cli.path_board.display()))?;
    let board: Board = board
        .parse()
        .with_context(|| format!("問題ファイル '{}' のパースに失敗", cli.path_board.display()))?;

    let solution = std::fs::read_to_string(&cli.path_solution)
        .with_context(|| format!("解ファイル '{}' を読めない", cli.path_solution.display()))?;
    let (score, solution) = parse_solution(&solution).with_context(|| {
        format!(
            "解ファイル '{}' のパースに失敗",
            cli.path_solution.display()
        )
    })?;

    let score_actual = verify_solution(&board, &solution).context("解の verify に失敗")?;

    ensure!(
        score == score_actual,
        "解のスコアが一致しない (file={score}, actual={score_actual})"
    );

    Ok(())
}

fn parse_solution(s: &str) -> anyhow::Result<(Score, ActionHistory)> {
    let fields: Vec<_> = s.split('\t').collect();
    ensure!(
        fields.len() == 2,
        "解ファイルはタブ区切りでちょうど 2 つのフィールドを持たねばならない"
    );

    let score: Score = fields[0]
        .parse()
        .with_context(|| format!("スコアのパースに失敗: '{}'", fields[0]))?;
    let solution: ActionHistory = fields[1]
        .parse()
        .with_context(|| format!("手順のパースに失敗: '{}'", fields[1]))?;

    Ok((score, solution))
}
