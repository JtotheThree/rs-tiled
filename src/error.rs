use crate::InvalidTilesetError::InvalidTileDimensions;
use std::num::ParseIntError;
use std::{fmt, path::PathBuf};

/// Errors that can occur while decoding csv data.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum CsvDecodingError {
    /// An error occurred when parsing tile data from a csv encoded dataset.
    TileDataParseError(ParseIntError),
}

impl fmt::Display for CsvDecodingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CsvDecodingError::TileDataParseError(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for CsvDecodingError {}

/// Errors that can occur parsing a Tileset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum InvalidTilesetError {
    /// An invalid width or height (0) dimension was found in the input.
    InvalidTileDimensions,
}

impl fmt::Display for InvalidTilesetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            InvalidTileDimensions => write!(
                f,
                "An invalid width or height (0) dimension was found in the input."
            ),
        }
    }
}

impl std::error::Error for InvalidTilesetError {}

/// Errors which occurred when parsing the file
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// A attribute was missing, had the wrong type of wasn't formated
    /// correctly.
    MalformedAttributes(String),
    /// An error occurred when decompressing using the
    /// [flate2](https://github.com/alexcrichton/flate2-rs) crate.
    DecompressingError(std::io::Error),
    /// An error occurred when decoding a base64 encoded dataset.
    Base64DecodingError(base64::DecodeError),
    /// An error occurred when decoding a csv encoded dataset.
    CsvDecodingError(CsvDecodingError),
    /// An error occurred when parsing an XML file, such as a TMX or TSX file.
    XmlDecodingError(xml::reader::Error),
    #[cfg(feature = "world")]
    /// An error occurred when attempting to deserialize a JSON file.
    JsonDecodingError(serde_json::Error),
    #[cfg(feature = "world")]
    /// No regex captures were found.
    CapturesNotFound,
    /// The XML stream ended before the document was fully parsed.
    PrematureEnd(String),
    /// The path given is invalid because it isn't contained in any folder.
    PathIsNotFile,
    /// An error generated by [`ResourceReader`](crate::ResourceReader) while trying to read a
    /// resource.
    ResourceLoadingError {
        /// The path to the file that was unable to be opened.
        path: PathBuf,
        /// The error that occurred when trying to open the file.
        err: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    /// There was an invalid tile in the map parsed.
    InvalidTileFound,
    /// Unknown encoding or compression format or invalid combination of both (for tile layers)
    InvalidEncodingFormat {
        /// The `encoding` attribute of the tile layer data, if any.
        encoding: Option<String>,
        /// The `compression` attribute of the tile layer data, if any.
        compression: Option<String>,
    },
    /// There was an error parsing the value of a [`PropertyValue`].
    ///
    /// [`PropertyValue`]: crate::PropertyValue
    InvalidPropertyValue {
        /// A description of the error that occurred.
        description: String,
    },
    /// Found an unknown property value type while parsing a [`PropertyValue`].
    ///
    /// [`PropertyValue`]: crate::PropertyValue
    UnknownPropertyType {
        /// The name of the type that isn't recognized by the crate.
        /// Supported types are `string`, `int`, `float`, `bool`, `color`, `file` and `object`.
        type_name: String,
    },
    /// A template was found that does not have an object element in it.
    TemplateHasNoObject,
    /// Found a WangId that was not properly formatted.
    InvalidWangIdEncoding {
        /// Stores the wrongly parsed String.
        read_string: String,
    },
    /// There was an error parsing an Object's data.
    InvalidObjectData {
        /// A description of the error that occurred.
        description: String,
    },
    /// There was an invalid tileset in the map parsed.
    InvalidTileset(InvalidTilesetError),
}

/// A result with an error variant of [`crate::Error`].
pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> std::result::Result<(), fmt::Error> {
        match self {
            Error::MalformedAttributes(s) => write!(fmt, "{}", s),
            Error::DecompressingError(e) => write!(fmt, "{}", e),
            Error::Base64DecodingError(e) => write!(fmt, "{}", e),
            Error::CsvDecodingError(e) => write!(fmt, "{}", e),
            Error::XmlDecodingError(e) => write!(fmt, "{}", e),
            #[cfg(feature = "world")]
            Error::JsonDecodingError(e) => write!(fmt, "{}", e),
            #[cfg(feature = "world")]
            Error::CapturesNotFound => write!(fmt, "No captures found in pattern"),
            Error::PrematureEnd(e) => write!(fmt, "{}", e),
            Error::PathIsNotFile => {
                write!(
                    fmt,
                    "The path given is invalid because it isn't contained in any folder."
                )
            }
            Error::ResourceLoadingError { path, err } => {
                write!(
                    fmt,
                    "Could not open '{}'. Error: {}",
                    path.to_string_lossy(),
                    err
                )
            }
            Error::InvalidTileFound => write!(fmt, "Invalid tile found in map being parsed"),
            Error::InvalidEncodingFormat { encoding: None, compression: None } =>
                write!(
                    fmt,
                    "Deprecated combination of encoding and compression"
                ),
            Error::InvalidEncodingFormat { encoding, compression } =>
                write!(
                    fmt,
                    "Unknown encoding or compression format or invalid combination of both (for tile layers): {} encoding with {} compression",
                    encoding.as_deref().unwrap_or("no"),
                    compression.as_deref().unwrap_or("no")
                ),
            Error::InvalidPropertyValue{description} =>
                write!(fmt, "Invalid property value: {}", description),
            Error::UnknownPropertyType { type_name } =>
                write!(fmt, "Unknown property value type '{}'", type_name),
            Error::TemplateHasNoObject => write!(fmt, "A template was found with no object element"),
            Error::InvalidWangIdEncoding{read_string} =>
                write!(fmt, "\"{}\" is not a valid WangId format", read_string),
            Error::InvalidObjectData{description} =>
                write!(fmt, "Invalid object data: {}", description),
            Error::InvalidTileset(e) => write!(fmt, "{}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::DecompressingError(e) => Some(e as &dyn std::error::Error),
            Error::Base64DecodingError(e) => Some(e as &dyn std::error::Error),
            Error::XmlDecodingError(e) => Some(e as &dyn std::error::Error),
            Error::ResourceLoadingError { err, .. } => Some(err.as_ref()),
            _ => None,
        }
    }
}
