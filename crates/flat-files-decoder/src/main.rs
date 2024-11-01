use flat_files_decoder::cli::run;
use std::process::ExitCode;
use tracing::{error, level_filters::LevelFilter, subscriber::set_global_default};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[tokio::main]
async fn main() -> ExitCode {
    init_tracing();
    if let Err(e) = run().await {
        error!("Decoder error: {e}");
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn init_tracing() {
    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    let subscriber_builder: tracing_subscriber::fmt::SubscriberBuilder<
        tracing_subscriber::fmt::format::DefaultFields,
        tracing_subscriber::fmt::format::Format,
        EnvFilter,
    > = FmtSubscriber::builder().with_env_filter(filter);
    set_global_default(subscriber_builder.with_ansi(true).pretty().finish()).expect(
        "Failed to set up the global default subscriber for logging. Please check if the RUST_LOG environment variable is set correctly.",
    );
}
