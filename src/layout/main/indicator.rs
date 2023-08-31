use tui::{
    backend::Backend,
    layout::Rect,
    Frame, widgets::Block, text::{Spans, Span}, style::{Style, Color},
};
use crate::{App, app::InstallStatus};

pub fn indicator_ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut App) {
    let line = match app.install_status {
        InstallStatus::NotInstalled => String::from("Environment not found. Press F6 to install"),
        InstallStatus::Installing(timestamp) => {
            String::from("Installing flash environment... ") + timestamp.elapsed().unwrap().as_secs().to_string().as_str() + " sec(s)"
        },
        InstallStatus::Installed => String::from("OK"),
    };

    let style = match app.install_status {
        InstallStatus::NotInstalled => Style::default().fg(Color::White).bg(Color::Red),
        InstallStatus::Installing(_) => Style::default().fg(Color::Black).bg(Color::Yellow),
        InstallStatus::Installed => Style::default().fg(Color::White).bg(Color::Green),
    };


    let title = Spans::from(Span::styled(line, style));
    let block = Block::default().title(title);
    f.render_widget(block, size);
}