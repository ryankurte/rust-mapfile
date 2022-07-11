use mapfile::*;

const EXAMPLES: &[&str] = &[
    "maps/partial.map",
    "maps/trezor.map",
];

#[test]
fn parse_examples() {
    for e in EXAMPLES {
        // Read in mapfile
        let d = std::fs::read_to_string(e).unwrap();
        // Attempt to parse
        let _m = MapFile::parse(&d).unwrap();
    }
}
