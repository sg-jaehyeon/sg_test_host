use crossterm::event::{KeyEvent, KeyCode};
use tui::{
    backend::Backend,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, layout::{Margin, Alignment},
};
use crate::App;

pub fn quit_ui<B: Backend>(f: &mut Frame<B>, _app: &mut App) {
    let title = Block::default()
                            .title("Dialog")
                            .borders(Borders::ALL);
    let size = f.size();
    let inner_size = f.size().inner(&Margin { vertical: size.height / 3, horizontal: size.width / 3 });
    let text_size = inner_size.inner(&Margin { vertical: inner_size.height / 3, horizontal: 2, });
    let paragraph = Paragraph::new("Are you sure you want to quit? [Y/n]")
                                            .alignment(Alignment::Center)
                                            .wrap(Wrap { trim: false });

    f.render_widget(paragraph, text_size);
    f.render_widget(title, inner_size);
}

pub fn control(app: &mut App, key: KeyEvent) -> Option<()> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            return Some(());
        },
        KeyCode::Char('n') | KeyCode::Char('N') => {
            app.index = 0;
        },
        _ => {
            
        },
    }
    None
}