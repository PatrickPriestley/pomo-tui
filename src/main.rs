use pomo_tui::tui::App;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize and run the TUI application
    let mut app = App::new()?;
    app.run().await?;
    Ok(())
}
