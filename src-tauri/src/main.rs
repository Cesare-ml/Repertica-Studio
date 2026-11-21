#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    // `Repertica Studio --mcp` runs the local read-only MCP server over STDIO instead of the window.
    if arguments.first().map(String::as_str) == Some("--mcp") {
        std::process::exit(repertica_studio_lib::mcp::main(&arguments[1..]));
    }
    repertica_studio_lib::run();
}
