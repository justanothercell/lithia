use std::fmt::{Debug, Formatter};
use std::fs::File;
use std::io::Read;
use std::string::ParseError;

#[derive(PartialEq)]
pub(crate) struct Source {
    st: SourceType,
    source: String
}

impl Debug for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Source({:?})", self.st)
    }
}

impl Source {
    pub(crate) fn from_file(path: String) -> Result<Self, ParseError> {
        Ok(Self {
            st: SourceType::File(path.clone()),
            source: {
                let mut f = File::open(path)?;
                let mut buffer = String::new();
                f.read_to_string(&mut buffer)?;
                buffer
            }
        })
    }

    pub(crate) fn from_string(source: String) -> Self{
        Self {
            st: SourceType::String,
            source
        }
    }
}