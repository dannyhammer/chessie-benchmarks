use std::{
    fs::{self, File},
    path::Path,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Result};
use csv::Writer;
use shakmaty::Position;

type EpdEntry = (String, Vec<u64>);

pub fn bench<const PRINT: bool>(epd: &[EpdEntry]) -> Result<()> {
    // let _chessie = chessie::Game::run_bench::<PRINT>(epd)?;

    // let _shakmaty = shakmaty::Chess::run_bench::<PRINT>(epd)?;

    // let _cozy_chess = cozy_chess::Board::run_bench::<PRINT>(epd)?;

    // let _chess = chess::Board::run_bench::<PRINT>(epd)?;

    const CHESSIE_VERSION: &str = "0.1.0";
    let csv_file_name = format!("chessie-{CHESSIE_VERSION}.csv");
    let mut writer = Writer::from_path(Path::new("benchmarks").join(csv_file_name))?;
    writer.write_record(&["name", "test", "fen", "nodes", "time", "nps", "m_nps"])?;

    for (i, (fen, depths)) in epd.into_iter().enumerate() {
        let _shakmaty = shakmaty::Chess::run_bench_on::<PRINT>(i, fen, depths, &mut writer)?;

        let _cozy_chess = cozy_chess::Board::run_bench_on::<PRINT>(i, fen, depths, &mut writer)?;

        let _chess = chess::Board::run_bench_on::<PRINT>(i, fen, depths, &mut writer)?;

        let _chessie = chessie::Game::run_bench_on::<PRINT>(i, fen, depths, &mut writer)?;
    }

    writer.flush()?;
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

    /*
    fn run_bench<const PRINT: bool>(epd: &[EpdEntry]) -> Result<(Duration, u64)> {
        let name = Self::name();
        let mut elapsed = Duration::default();
        let mut nodes = 0;

        for (fen, depths) in epd {
            let (e, n) = Self::run_bench_on::<PRINT>(fen, depths)?;
            elapsed += e;
            nodes += n;
        }

        // if PRINT {
        eprintln!("{name} finished bench suite in {elapsed:?}");
        // }

        Ok((elapsed, nodes))
    }
     */

    fn run_bench_on<const PRINT: bool>(
        test_num: usize,
        fen: &str,
        depths: &[u64],
        writer: &mut Writer<File>,
    ) -> Result<(Duration, u64)> {
        let name = Self::name();
        let mut nodes = 0;
        let now = Instant::now();
        let board = Self::from_fen(&fen)?;
        if PRINT {
            eprint!("\nRunning {name} on {test_num}: {fen}\n\tDepth: ");
        }

        for depth in depths {
            if PRINT {
                eprint!("{depth} ");
            }

            nodes += perft::<false>(board.clone(), *depth);
        }

        if PRINT {
            eprintln!();
        }

        let elapsed = now.elapsed();
        let nps = nodes as f32 / elapsed.as_secs_f32();
        let m_nps = nps / 1_000_000.0;

        writer.write_record(&[
            name.to_string(),
            test_num.to_string(),
            fen.to_string(),
            nodes.to_string(),
            elapsed.as_secs_f32().to_string(),
            nps.to_string(),
            m_nps.to_string(),
        ])?;

        // if PRINT {
        // eprintln!("{name} finished position in {elapsed:?}");
        // }
        Ok((elapsed, nodes))
    }
}

impl Chessboard for chessie::Game {
    type Move = chessie::Move;

    fn name() -> &'static str {
        "chessie 0.1.0"
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
        "chess 3.2.0"
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
        "cozy-chess 0.3.4"
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
        "shakmaty 0.27.2"
    }

    fn from_fen(fen: &str) -> Result<Self> {
        fen.parse::<shakmaty::fen::Fen>()?
            .into_position(shakmaty::CastlingMode::Standard)
            .map_err(|e| anyhow!("shakmaty failed on {fen:?}:\n{e}"))
    }

    fn new_with_move_made(mut self, mv: Self::Move) -> Self {
        self.play_unchecked(&mv);
        self
    }

    fn legal_moves(&self) -> impl IntoIterator<Item = Self::Move> {
        shakmaty::Position::legal_moves(self)
    }
}
