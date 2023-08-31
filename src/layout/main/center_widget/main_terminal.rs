use crossterm::event::KeyEvent;
use tui::{
    backend::Backend,
    layout::{Rect, Margin},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame
};
use crate::App;

pub fn main_terminal_ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut App) {
    let block = Block::default()
                        .title("TERMINAL")
                        .borders(Borders::ALL);

    main_terminal_inner_ui(f, block.inner(size), app);
    f.render_widget(block, size);
}

fn main_terminal_inner_ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut App) {
    let output = app.main_terminal.output();
    let mut paragraph = Paragraph::new(output).wrap(Wrap { trim: false });
    paragraph = paragraph.scroll((
        if output.lines().count() as u16 >= size.height {
            output.lines().count() as u16 - size.height
        } else {
            0
        }, 0)
    );

    f.render_widget(paragraph, size.inner(&Margin { vertical: 0, horizontal: 1, }));
}

pub fn control(_app: &mut App, key: KeyEvent) {
    match key.code {
        _ => {},
    }
}