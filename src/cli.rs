use clap::Parser;

#[derive(Parser)]
#[command(version)]
pub struct Cli {
    /// TUI render interval in milliseconds
    #[arg(short, long, default_value = "1000")]
    pub tick: u64,
    /// Timer names (length must match `--durations`)
    #[arg(default_values = ["Work", "Break"], num_args(1..), short, long)]
    pub names: Vec<String>,
    /// Timer durations (length must match `--names`)
    #[arg(default_values = ["25m", "5m"], num_args(1..), short, long)]
    pub durations: Vec<String>,
    /// Limit number of cycles, 0 to set no limits
    #[arg(default_value = "0", short, long)]
    pub cycles: u64,
    /// Disable notifications
    #[arg(default_value_t = false, short = 'N')]
    pub no_notify: bool,
}
