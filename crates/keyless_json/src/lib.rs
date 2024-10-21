pub use serializer::{to_string, to_writer, Serializer};
pub use string::write_escaped_str_contents;

mod error;
mod serializer;
mod string;
