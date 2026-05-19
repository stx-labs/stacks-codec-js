use std::fmt::Display;

#[derive(Debug)]
pub struct DeserializeError {
    pub error: String,
}

impl DeserializeError {
    pub fn as_string(self) -> String {
        self.error
    }
}

impl From<String> for DeserializeError {
    fn from(error: String) -> Self {
        DeserializeError { error }
    }
}

impl From<std::io::Error> for DeserializeError {
    fn from(err: std::io::Error) -> Self {
        format!("Serialization error: {:?}", err).into()
    }
}

impl Display for DeserializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.error)
    }
}
