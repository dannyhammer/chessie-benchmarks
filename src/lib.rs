use std::{
    fs,
    path::Path,
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use shakmaty::{FromSetup, Position};

type EpdEntry = (String, Vec<u64>);

pub fn bench<const PRINT: bool>(epd: &[EpdEntry]) -> Result<()> {
    let _shakmaty = shakmaty::Chess::run_bench::<PRINT>(epd)?;

    let _cozy_chess = cozy_chess::Board::run_bench::<PRINT>(epd)?;

    let _chess = chess::Board::run_bench::<PRINT>(epd)?;

    let _chessie = chessie::Game::run_bench::<PRINT>(epd)?;
    Ok(())
}

pub fn run_bench_on<const PRINT: bool>(fen: &str, depths: &[u64]) -> Result<()> {
    let _shakmaty = shakmaty::Chess::run_bench_on::<PRINT>(fen, depths)?;

    let _cozy_chess = cozy_chess::Board::run_bench_on::<PRINT>(fen, depths)?;

    let _chess = chess::Board::run_bench_on::<PRINT>(fen, depths)?;

    let _chessie = chessie::Game::run_bench_on::<PRINT>(fen, depths)?;
    Ok(())
}

pub fn parse_epd(epd_file: impl AsRef<Path>) -> Result<Vec<EpdEntry>> {
    let mut epd_entries = Vec::with_capacity(128); // Number of entries in standard.epd
    let contents = fs::read_to_string(epd_file)?;
    for epd in contents.lines() {
        let mut parts = epd.split(';');

        // Extract the FEN
        let fen = parts.next().unwrap().trim().to_string();

        let mut perft_parts = Vec::with_capacity(8); // Maximum depth we'll ever search is 6, so this is fine
        for perft_data in parts {
            // Extract and parse the depth and expected result
            let depth = perft_data.get(1..2).unwrap().trim().parse()?;
            // let expected = perft_data.get(3..).unwrap().trim().parse()?;

            perft_parts.push(depth);
        }

        epd_entries.push((fen, perft_parts));
    }
    Ok(epd_entries)
}

fn perft<const BULK: bool>(board: impl Chessboard, depth: u64) -> u64 {
    // Recursion limit; return 1, since we're fathoming this node.
    if depth == 0 {
        return 1;
    }

    // Recursively accumulate the nodes from the remaining depths
    board.legal_moves().into_iter().fold(0, |nodes, mv| {
        nodes + perft::<BULK>(board.clone().new_with_move_made(mv), depth - 1)
    })
}

pub trait Chessboard
where
    Self: Sized + Clone,
{
    type Move;
    fn name() -> &'static str;
    fn from_fen(fen: &str) -> Result<Self>;
    fn new_with_move_made(self, mv: Self::Move) -> Self;
    // fn make_move(&mut self, mv: Self::Move);
    fn legal_moves(&self) -> impl IntoIterator<Item = Self::Move>;

    fn run_bench<const PRINT: bool>(epd: &[EpdEntry]) -> Result<Duration> {
        let name = Self::name();
        let mut elapsed = Duration::default();

        for (fen, depths) in epd {
            elapsed += Self::run_bench_on::<PRINT>(fen, depths)?;
        }

        // if PRINT {
        eprintln!("{name} finished bench suite in {elapsed:?}");
        // }

        Ok(elapsed)
    }

    fn run_bench_on<const PRINT: bool>(fen: &str, depths: &[u64]) -> Result<Duration> {
        let name = Self::name();
        let now = Instant::now();
        let board = Self::from_fen(&fen)?;
        if PRINT {
            eprint!("\nRunning {name} on {fen}\n\tDepth: ");
        }

        for depth in depths {
            if PRINT {
                eprint!("{depth} ");
            }

            perft::<false>(board.clone(), *depth);
        }

        if PRINT {
            eprintln!();
        }

        let elapsed = now.elapsed();
        // if PRINT {
        // eprintln!("{name} finished position in {elapsed:?}");
        // }
        Ok(elapsed)
    }
}

impl Chessboard for chessie::Game {
    type Move = chessie::Move;

    fn name() -> &'static str {
        "chessie"
    }

    fn from_fen(fen: &str) -> Result<Self> {
        chessie::Game::from_fen(fen)
    }

    fn new_with_move_made(self, mv: Self::Move) -> Self {
        chessie::Game::with_move_made(&self, mv)
    }

    fn legal_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        chessie::Game::get_legal_moves(&self)
    }
}

impl Chessboard for chess::Board {
    type Move = chess::ChessMove;

    fn name() -> &'static str {
        "chess"
    }

    fn from_fen(fen: &str) -> Result<Self> {
        fen.parse().map_err(|e| anyhow!("{e}"))
    }

    fn new_with_move_made(self, mv: Self::Move) -> Self {
        chess::Board::make_move_new(&self, mv)
    }

    fn legal_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        chess::MoveGen::new_legal(self)
    }
}

impl Chessboard for cozy_chess::Board {
    type Move = cozy_chess::Move;

    fn name() -> &'static str {
        "cozy-chess"
    }

    fn from_fen(fen: &str) -> Result<Self> {
        fen.parse().map_err(|e| anyhow!("{e}"))
    }

    fn new_with_move_made(mut self, mv: Self::Move) -> Self {
        self.play_unchecked(mv);
        self
    }

    fn legal_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        let mut moves = Vec::with_capacity(218);
        self.generate_moves(|m| {
            moves.extend(m);
            false
        });
        moves
    }
}

impl Chessboard for shakmaty::Chess {
    type Move = shakmaty::Move;

    fn name() -> &'static str {
        "shakmaty"
    }

    fn from_fen(fen: &str) -> Result<Self> {
        let setup = shakmaty::fen::Fen::from_str(fen)?;
        shakmaty::Chess::from_setup(setup.into(), shakmaty::CastlingMode::Standard)
            .map_err(|e| anyhow!("{e}"))
    }

    fn new_with_move_made(mut self, mv: Self::Move) -> Self {
        self.play_unchecked(&mv);
        self
    }

    fn legal_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        shakmaty::Position::legal_moves(self)
    }
}
