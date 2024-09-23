use chessie_benchmarks::*;

fn main() {
    let epd = parse_epd("src/standard.epd").unwrap();
    let epd = &epd[..2];

    bench::<true>(&epd).unwrap()
}
