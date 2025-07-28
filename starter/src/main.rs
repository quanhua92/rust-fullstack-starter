use starter::cli::CliApp;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    CliApp::run().await
}
