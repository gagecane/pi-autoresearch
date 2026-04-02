use anyhow::Result;
use clap::Parser;
use pi_autoresearch::cli::Cli;
use pi_autoresearch::orchestrator::ExperimentOrchestrator;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut orchestrator = ExperimentOrchestrator::new(cli);
    orchestrator.run().await
}
