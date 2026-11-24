// Keep the proposed native contract compiled and exercised by the existing
// cargo test workflow without mounting any commands or touching core lib.rs.
#[path = "../src/auction_pkce.rs"]
mod auction_pkce;
