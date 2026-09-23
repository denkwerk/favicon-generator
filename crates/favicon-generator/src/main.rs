use clap::Parser;

fn main() -> anyhow::Result<()> {
    favicon_generator::run(favicon_generator::cli::Cli::parse())
}
