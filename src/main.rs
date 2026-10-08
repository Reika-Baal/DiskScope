
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.iter().skip(1).any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    println!("====================================");
    println!("             DiskScope              ");
    println!("====================================");

    println!("A disk space analyser built in Rust.");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    println!();

    println!("Welcome to DiskScope!");
    println!("Directory scanning coming soon...");
}

fn print_help() {
    println!("DiskScope - Disk Space Analyser");
    println!();
    println!("USAGE:");
    println!("    diskscope [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help    Display this help information");
    println!();
    println!("More functionality coming soon!");
}
