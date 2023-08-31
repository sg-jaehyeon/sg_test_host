use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};
use tui::style::{Style, Color};
use tui::text::{Spans, Span};
use tui::widgets::ListItem;
use crossterm::event::{self, Event};
use tui::{
    backend::Backend,
    Terminal,
};
use std::sync::mpsc::{self, Sender, Receiver};
use tokio::sync::mpsc::{self as async_mpsc, Sender as AsyncSender, Receiver as AsyncReceiver};

use crate::test::env_setup::check_env;

use super::ui_selection::*;
use super::jetson::*;
use super::logger::Logger;

#[derive(PartialEq)]
pub enum InstallStatus {
    NotInstalled,
    Installing(SystemTime),
    Installed,
}

pub struct App<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
    pub selection: UISelectionModel,
    pub devlist: Vec<Jetson>,
    pub installer: Option<JoinHandle<()>>,
    pub flash_handle: Option<JoinHandle<()>>,
    pub main_terminal: Logger,
    pub refreshing: bool,
    pub install_status: InstallStatus,
    pub flash_status: FlashStatus,
    pub async_tx: AsyncSender<Signal>,
    pub async_rx: AsyncReceiver<Signal>,
    pub tx: Sender<Signal>,
    pub rx: Receiver<Signal>,
}

impl<'a> App<'a> {
    pub fn new() -> App<'a> {
        let (async_tx, async_rx) = async_mpsc::channel(4096);
        let (tx, rx) = mpsc::channel();
        App {
            titles: vec!["Main", "ModeSelect", "Quit"],
            index: 0,
            selection: UISelectionModel { focused: UISelection::DeviceList(None), current: UISelection::DeviceList(None) },
            devlist: vec![],
            installer: None,
            flash_handle: None,
            main_terminal: Logger::new("Main"),
            refreshing: false,
            install_status: InstallStatus::NotInstalled,
            flash_status: FlashStatus::Wait,
            async_tx,
            async_rx,
            tx,
            rx,
        }
    }

    pub fn selected_device_index(&self) -> Option<usize> {
        if let UISelection::DeviceList(Some(index)) = self.selection.current {
            return Some(index);
        }
        None
    }

    pub fn create_new_async_publisher(&self) -> AsyncSender<Signal> {
        self.async_tx.clone()
    }

    pub fn create_new_publisher(&self) -> Sender<Signal> {
        self.tx.clone()
    }

    pub fn has_flashing_device(&self) -> bool {
        self.devlist.iter().any(|jetson| {
            jetson.is_flashing()
        })
    }

    pub fn next_device(&mut self) -> Option<usize> {
        if let UISelection::DeviceList(Some(index)) = self.selection.current {
            if index + 1 >= self.devlist.len() {
                self.select(UISelectionModel{ focused: self.selection.focused, current: UISelection::DeviceList(Some(0)) });
                return Some(0);
            } else {
                self.select(UISelectionModel{ focused: self.selection.focused, current: UISelection::DeviceList(Some(index + 1)) });
                return Some(index + 1);
            }
        }
        None
    }

    pub fn previous_device(&mut self) -> Option<usize> {
        if let UISelection::DeviceList(Some(index)) = self.selection.current {
            if index == 0 {
                self.select(UISelectionModel{ focused: self.selection.focused, current: UISelection::DeviceList(Some(self.devlist.len() - 1)) });
                return Some(self.devlist.len() - 1);
            } else {
                self.select(UISelectionModel{ focused: self.selection.focused, current: UISelection::DeviceList(Some(index - 1)) });
                return Some(index - 1);
            }
        }
        None
    }

    pub fn get_device_from_instance_number(&mut self, instance_number: &str) -> Option<&mut Jetson> {
        let result = self.devlist.iter_mut().find(|jetson| {
            jetson.instance_number == instance_number
        });
        match result {
            Some(jetson) => {
                Some(jetson)
            },
            _ => {
                None
            }
        }
    }

    pub fn clear_devlist(&mut self) {
        let mut filtered = vec![];

        while !self.devlist.is_empty() {
            let item = self.devlist.pop().unwrap();
            if item.is_flashed() || item.is_flashing() {
                filtered.push(item);
            }
        }
        
        while !filtered.is_empty() {
            self.devlist.push(filtered.pop().unwrap());
        }
    }

    pub fn select(&mut self, new: UISelectionModel) {
        self.selection = new;
    }

    pub fn change_focused(&mut self, focused: UISelection) {
        self.selection.focused = focused;
    }

    pub fn change_current(&mut self, current: UISelection) {
        self.selection.current = current;
    }

    pub fn list(&self) -> Vec<ListItem> {

        let mut ret = vec![];

        for (index, jetson) in self.devlist.iter().enumerate() {

            let line = jetson.to_string();
            
            let style = match self.selection.current {
                UISelection::DeviceList(Some(dev_index)) if index == dev_index => {
                    Style::default().fg(Color::Black).bg(Color::White)
                },
                _ => {
                    Style::default()
                }
            };

            ret.push(ListItem::new(Spans::from(Span::styled(line, style))));
        }

        ret
    }

}

pub async fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App<'static>) -> Result<(), Box<dyn std::error::Error>> {
    // 최초 1회 디바이스 리스트 초기화
    use super::module_detect::refresh_devlist;
    app.main_terminal.init();

    refresh_devlist(&mut app);
    check_env(app.tx.clone());

    loop {
        // 플래싱 완료 확인
        let flash_handle = app.flash_handle.take();
        if let Some(handle) = flash_handle {
            if handle.is_finished() {
                handle.join().unwrap();
                app.tx.send(Signal::Message(String::from("Flashing complete\n"))).unwrap();
                app.flash_status = FlashStatus::Finished;
            } else {
                app.flash_handle = Some(handle);
            }
        } else {
            app.flash_handle = None;
        }

        // 시그널 핸들링
        loop {
            if let Ok(sig) = app.rx.try_recv() {
                match sig {
                    Signal::Message(msg) => {
                        app.main_terminal.create_new_publisher().send(msg).unwrap();
                    },
                    Signal::EnvironmentInstalled => {
                        app.install_status = InstallStatus::Installed;
                    },
                    Signal::EnvironmentInstalling(timestamp) => {
                        app.install_status = InstallStatus::Installing(timestamp);
                    }
                    _ => {}
                }
            } else {
                break;
            }
        }

        for dev in &mut app.devlist {
            if dev.logger.is_some() {
                dev.logger.as_mut().unwrap().output();
            }
        }

        // APP 메인 루프 : main_ui 레이아웃에 맞춰 프레임 렌더링
        match app.index {
            0 => {
                terminal.draw(|f| super::layout::main::main_ui(f, &mut app))?;
            },
            1 => {
                terminal.draw(|f| super::layout::select_mode::select_mode_ui(f, &mut app))?;
            }
            2 => {
                terminal.draw(|f| super::layout::quit::quit_ui(f, &mut app))?;
            },
            _ => {},
        }

        // 입력값 체크
        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match app.index {
                    0 => {
                        if let Some(()) = super::layout::main::control(&mut app, key) {
                            return Ok(());
                        }
                    },
                    1 => {
                        if let Some(()) = super::layout::select_mode::control(&mut app, key) {
                            return Ok(());
                        }
                    }
                    2 => {
                        if let Some(()) = super::layout::quit::control(&mut app, key) {
                            return Ok(());
                        }
                    },
                    _ => {},
                }
            }
        }
    }
}