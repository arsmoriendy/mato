use clap::Parser;

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// TUI render interval in milliseconds
    #[arg(short, long, default_value = "1000")]
    pub tick: u64,
    /// timer names (length must match `--durations`)
    #[arg(required(true), short, long)]
    pub names: Vec<String>,
    /// timer durations in milliseconds (length must match `--names`)
    #[arg(required(true), short, long)]
    pub durations: Vec<u64>,
}
