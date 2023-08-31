use crossterm::{event::{KeyCode, KeyEvent}};
use tui::{
    backend::Backend,
    layout::{Rect, Margin, Alignment},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame
};
use crate::{App, UISelection};

pub fn terminal_ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut App) {
    match app.selection.current {
        UISelection::DeviceList(None) => {
            // There are no devices
            let border = Block::default().borders(Borders::ALL);
            let text_size = size.inner(&Margin { vertical: size.height / 3, horizontal: size.width / 3, });
            let paragraph = Paragraph::new("There are no devices.\n\nPlease connect your Jetson module with Linux host through the appropriate USB port and ensure that module is in recovery mode.").alignment(Alignment::Center).wrap(Wrap { trim : false } );

            f.render_widget(border, size);
            f.render_widget(paragraph, text_size);

            return;
        },
        UISelection::DeviceList(Some(index)) => {
            let terminal_name = app.devlist[index].logger.as_ref().unwrap().name.clone();
            let block = Block::default().title(&terminal_name[..]).borders(Borders::ALL);

            terminal_inner_ui(f,  block.inner(size), app);
            f.render_widget(block, size);
        }
        _ => { return; }
    }
}

fn terminal_inner_ui<B: Backend>(f: &mut Frame<B>, size: Rect, app: &mut App) {
    // let scroll = terminal.scroll;
    // let mut new_scroll = scroll;
    if let UISelection::DeviceList(Some(index)) = app.selection.current {
        let output = app.devlist[index].logger.as_mut().unwrap().output();

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
}

pub fn control(_app: &mut App, key: KeyEvent) {
    // let mut _selection: &mut UISelectionModel = &mut app.selection;
    // let terminal = app.current_terminal().1;
    match key.code {
        KeyCode::Up => {
            // terminal.scroll += 1;
        },
        KeyCode::Down => {
            // terminal.scroll -= 1;
        },
        KeyCode::Left => {

        },
        KeyCode::Right => {

        },
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
            // app.select(UISelectionModel { focused: UISelection::Main, current: UISelection::FlashTerminal });
        },
        KeyCode::Tab => {
            // 터미널 안에서 탭, 백탭을 누르면 해당 디바이스의 다음 터미널로 이동
            // match app.get_terminal_device() {
            //     Some((_, jetson)) => {
            //         jetson.next_logger();
            //     },
            //     None => {
            //         // 메인 로거에서는 아무 행동도 하지 않음
            //     }
            // }
        },
        KeyCode::BackTab => {
            // match app.get_terminal_device() {
            //     Some((_, jetson)) => {
            //         jetson.previous_logger();
            //     },
            //     None => {
            //         // 메인 로거에서는 아무 행동도 하지 않음
            //     }
            // }
        },
        KeyCode::Char('L') => {
            // terminal.clear();
        },
        KeyCode::Enter => {
            // if terminal.is_serialport() {
            //     terminal.write_and_read("\n".to_string().as_bytes().to_vec());
            // }
        },
        KeyCode::Char(_ch) => {
            // if terminal.is_serialport() {
            //     terminal.write_and_read(ch.to_string().as_bytes().to_vec());
            // }
        },
        _ => {

        }
    }
}