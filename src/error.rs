use thiserror::Error;
#[derive(Debug, Error)]
pub enum MyError {
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(std::num::ParseIntError),
    #[error("CSV read error: {0}")]
    CsvError(#[from] csv::Error),
}

// impl std::fmt::Display for MyError {
//     fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
//         match self {
//             | Self::IoError(err) => write!(f, "IO error: {err}"),
//             | Self::ParseError(err) => write!(f, "Parse error: {err}"),
//             | Self::CsvError(err) => write!(f, "Csv read error: {err}"),
//         }
//     }
// }

// impl std::convert::From<csv::Error> for MyError {
//     fn from(error: csv::Error) -> Self {
//         Self::CsvError(error)
//     }
// }

// impl std::error::Error for MyError {}
