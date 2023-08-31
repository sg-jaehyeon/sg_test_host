#[derive(PartialEq, Clone, Copy)]
pub enum UISelection {
    Main,
    DeviceList(Option<usize>),
    MainTerminal,
    FlashTerminal,
    SelectMode,
    Quit,
}

#[derive(PartialEq, Clone, Copy)]
pub struct UISelectionModel {
    pub focused: UISelection,
    pub current: UISelection,
}