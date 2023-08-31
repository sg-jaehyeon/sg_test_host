use std::fmt;

pub struct DtbProperty {
    pub key: String,
    pub value: Option<String>,
}

impl fmt::Debug for DtbProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DtbProperty")
            .field("key", &self.key)
            .field("value", &self.value)
            .finish()
    }
}
