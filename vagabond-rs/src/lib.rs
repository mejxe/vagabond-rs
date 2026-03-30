pub mod ai;
pub mod board;
pub mod engine;
pub mod moves;
pub mod performance;
pub mod tt;
pub mod uci;

fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    s.as_bytes()
        .rchunks(3)
        .rev()
        .map(std::str::from_utf8)
        .collect::<Result<Vec<&str>, _>>()
        .unwrap()
        .join(".")
}
