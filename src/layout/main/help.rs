use tui::{
    backend::Backend,
    layout::Rect,
    Frame, widgets::{Paragraph, Wrap}, style::{Style, Color}, text::{Span, Spans},
};
use crate::App;

pub fn help_ui<B: Backend>(f: &mut Frame<B>, size: Rect, _app: &mut App) {
    let keys = vec![
        ("F5", "Refresh device list"),
        ("F6", "Install environment for flashing"),
        ("Q", "Quit"),
        ("↑ ↓ ", "Select device"),
        ("ENTER", "Flash device"),
    ];
    let key_style = Style::default().bg(Color::White).fg(Color::Black);
    let description_style = Style::default();
    
    let mut spans = Spans::from(vec![]);

    for key in keys {
        spans.0.push(Span::styled(key.0, key_style));
        spans.0.push(Span::styled(" ", Style::default()));
        spans.0.push(Span::styled(key.1, description_style));
        spans.0.push(Span::styled(" ", Style::default()));
    }

    let paragraph = Paragraph::new(spans).wrap(Wrap { trim: false });
    
    f.render_widget(paragraph, size);
}