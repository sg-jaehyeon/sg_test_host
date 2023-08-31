use crossterm::event::{KeyEvent, KeyCode};
use tui::{
    backend::Backend,
    widgets::{Block, Borders, Paragraph},
    Frame, layout::{Margin, Alignment},
};
use crate::{App, test::{flash::flash_device, env_setup::{check_env, setup_workspace}}};

pub fn select_mode_ui<B: Backend>(f: &mut Frame<B>, _app: &mut App) {
    let title = Block::default()
                            .title("Dialog")
                            .borders(Borders::ALL);
    let size = f.size();
    let inner_size = f.size().inner(&Margin { vertical: size.height / 3, horizontal: size.width / 3 });
    let text_size = inner_size.inner(&Margin { vertical: inner_size.height / 3, horizontal: 1, });
    let paragraph = Paragraph::new("Select flashing mode\n\n[1] Flashing for test\n[2] Flashing for release\n[Q] Return to device list")
                                            .alignment(Alignment::Center);

    f.render_widget(paragraph, text_size);
    f.render_widget(title, inner_size);
}

pub fn control(app: &mut App, key: KeyEvent) -> Option<()> {
    match key.code {
        KeyCode::Char('1') | KeyCode::Char('t') | KeyCode::Char('T') | KeyCode::Enter => {
            // test
            app.index = 0;
            if check_env(app.tx.clone()) {
                flash_device(app, true).unwrap();
            } else {
                setup_workspace(app.tx.clone(), ".");
            }
        }
        KeyCode::Char('2') | KeyCode::Char('r') | KeyCode::Char('R') => {
            // release
            app.index = 0;
            if check_env(app.tx.clone()) {
                flash_device(app, false).unwrap();   
            } else {
                setup_workspace(app.tx.clone(), ".");
            }
        }
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => {
            app.index = 0;
        },
        _ => {},
    }
    None
}