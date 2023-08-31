use std::sync::mpsc::{self, Sender, Receiver};

pub struct Logger {
    pub name: String,
    rx: Option<Receiver<String>>,
    tx: Option<Sender<String>>,
    pub scroll: isize,
    pub opened: bool,
    buffer: String,
}

impl Logger {
    pub fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
            rx: None,
            tx: None,
            scroll: 0,
            buffer: String::new(),
            opened: false,
        }
    }

    pub fn init(&mut self) {
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        self.tx = Some(tx);
    }

    pub fn create_new_publisher(&mut self) -> Sender<String> {
        self.tx.as_mut().unwrap().clone()
    }

    pub fn kill_tx(_tx: Sender<String>) { }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn close(&mut self) {
        self.opened = false;
    }
    
    pub fn open(&mut self) {
        self.opened = true;
    }

    pub fn output(&mut self) -> &str {
        let rx_as_ref = self.rx.as_ref().unwrap();

        loop {
            match rx_as_ref.try_recv() {
                Ok(s) => {
                    self.buffer += &s[..];
                },
                _ => {
                    break;
                }
            }
        }
        while self.buffer.lines().count() > 60 {
            self.buffer.drain(..=self.buffer.find('\n').unwrap_or(0)+1);
        }

        &self.buffer
    }

}