use chessie_benchmarks::*;

fn main() {
    let epd = parse_epd("src/standard.epd").unwrap();
    // let epd = &epd[49..52];

    if let Err(e) = bench::<true>(&epd) {
        eprintln!("ERROR: {e}");
    }
}
