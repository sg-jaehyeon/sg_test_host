pub mod term;
pub mod layout;
pub mod jetson;
pub mod ui_selection;
pub mod app;
pub mod test;
pub mod devicetree;
pub mod logger;
pub mod module_detect;

use ui_selection::{UISelection, UISelectionModel};
use app::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use term::{term_init, term_deinit};

    let mut terminal = term_init()?;

    let app: App<'static> = App::new();
    let _ = run_app(&mut terminal, app).await?;

    term_deinit(terminal)?;

    Ok(())
}