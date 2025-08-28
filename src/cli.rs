use clap::Parser;

#[derive(Parser)]
#[command(version, about = "Automatic pomodoro TUI timer")]
pub struct Cli {
    /// TUI render interval in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    pub tick: u64,
    /// Name for each timer
    #[arg(default_values = ["Work", "Break"], num_args(1..), short, long)]
    pub names: Vec<String>,
    /// Duration for each timer, e.g. 3h2m1s
    #[arg(default_values = ["25m", "5m"], num_args(1..), short, long)]
    pub durations: Vec<String>,
    /// Limit number of cycles, 0 to set no limits
    #[arg(default_value_t = 0, short, long)]
    pub cycles: u64,
    /// Disable notifications
    #[arg(default_value_t = false, short = 'N')]
    pub no_notify: bool,
}
