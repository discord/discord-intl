use serde::{ser, Serialize};

use crate::error::{Error, Result};
use crate::string::write_escaped_str_contents;

macro_rules! write_byte {
    ($dest:expr, $byte:literal) => {{
        $dest.write_all($byte as &[u8])
    }};
}

pub struct Serializer<W> {
    writer: W,
}

#[inline]
fn write_array_start<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    write_byte!(writer, b"[")
}

#[inline]
fn write_element_separator<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    write_byte!(writer, b",")
}

#[inline]
fn write_array_end<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    write_byte!(writer, b"]")
}

#[inline]
fn write_object_start<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    write_byte!(writer, b"{")
}

#[inline]
fn write_object_separator<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    write_byte!(writer, b":")
}

#[inline]
fn write_object_end<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    write_byte!(writer, b"}")
}

#[inline]
fn write_string_delimiter<W: std::io::Write>(writer: &mut W) -> std::io::Result<()> {
    write_byte!(writer, b"\"")
}

// By convention, the public API of a Serde serializer is one or more `to_abc`
// functions such as `to_string`, `to_bytes`, or `to_writer` depending on what
// Rust types the serializer is able to produce as output.
pub fn to_writer<W, T>(writer: &mut W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: ?Sized + Serialize,
{
    let mut ser = Serializer { writer };
    value.serialize(&mut ser)
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut buffer = Vec::with_capacity(128);
    to_writer(&mut buffer, value)?;
    // The output is guaranteed to be UTF-8
    Ok(unsafe { String::from_utf8_unchecked(buffer) })
}

impl<'a, W: std::io::Write> ser::Serializer for &'a mut Serializer<W> {
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = ArraySerializer<'a, W>;
    type SerializeTuple = ArraySerializer<'a, W>;
    type SerializeTupleStruct = ArraySerializer<'a, W>;
    type SerializeTupleVariant = ArraySerializer<'a, W>;
    type SerializeMap = ArraySerializer<'a, W>;
    type SerializeStruct = ArraySerializer<'a, W>;
    type SerializeStructVariant = ArraySerializer<'a, W>;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<()> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i64(self, v: i64) -> Result<()> {
        let mut buffer = itoa::Buffer::new();
        Ok(self.writer.write_all(buffer.format(v).as_bytes())?)
    }

    fn serialize_u8(self, v: u8) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<()> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<()> {
        let mut buffer = itoa::Buffer::new();
        Ok(self.writer.write_all(buffer.format(v).as_bytes())?)
    }

    fn serialize_f32(self, v: f32) -> Result<()> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<()> {
        let mut buffer = ryu::Buffer::new();
        Ok(self.writer.write_all(buffer.format_finite(v).as_bytes())?)
    }

    fn serialize_char(self, v: char) -> Result<()> {
        // A char encoded as UTF-8 takes 4 bytes at most.
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<()> {
        write_string_delimiter(&mut self.writer)?;
        write_escaped_str_contents(&mut self.writer, v)?;
        write_string_delimiter(&mut self.writer)?;
        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<()> {
        Ok(self.writer.write_all(b"null" as &[u8])?)
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<()> {
        self.serialize_u32(variant_index)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `[<index>, <value>]`.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        write!(&mut self.writer, "[")?;
        // writer.write_all(ARRAY_START)?;
        self.serialize_u32(variant_index)?;
        write!(&mut self.writer, ",")?;
        // writer.write_all(ARRAY_SEPARATOR)?;
        value.serialize(&mut *self)?;
        write!(&mut self.writer, "]")?;
        // self.writer.write_all(ARRAY_END)?;
        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        write_array_start(&mut self.writer)?;
        Ok(ArraySerializer::new(self))
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in JSON as `[<index>, ...<data>]`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        write_array_start(&mut self.writer)?;
        self.serialize_u32(variant_index)?;
        write_element_separator(&mut self.writer)?;
        Ok(ArraySerializer::new(self))
    }

    // Maps in keyless JSON are preserved as JSON objects, since it's the most compact possible
    // representation of unknown fields and values.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        write_object_start(&mut self.writer)?;
        Ok(ArraySerializer::new(self))
    }

    // Structs are where keyless json is able to minify the most. The well-known fields and their
    // order in the data mean the entire struct can be written as a plain array. The deserializer
    // is responsible for parsing this into the appropriate field slots by index on the other side.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        write_array_start(&mut self.writer)?;
        Ok(ArraySerializer::new(self))
    }

    // Struct variants are represented in keyless JSON as `[<index>, <value>, <value>]`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        write_array_start(&mut self.writer)?;
        variant_index.serialize(&mut *self)?;
        write_element_separator(&mut self.writer)?;
        Ok(ArraySerializer::new(self))
    }
}

/// A wrapper type around Serializer that adds state for whether the first element of an array has
/// been written yet, allowing it to track whether a separator needs to be added before each new
/// element that is written.
pub struct ArraySerializer<'a, W: 'a> {
    serializer: &'a mut Serializer<W>,
    after_first: bool,
}

impl<'a, W: std::io::Write> ArraySerializer<'a, W> {
    fn new(serializer: &'a mut Serializer<W>) -> Self {
        Self {
            serializer,
            after_first: false,
        }
    }

    fn write_or_skip_first_separator(&mut self) -> Result<()> {
        if self.after_first {
            write_element_separator(&mut self.serializer.writer)?;
        } else {
            self.after_first = true;
        }
        Ok(())
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a, W: std::io::Write> ser::SerializeSeq for ArraySerializer<'a, W> {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_or_skip_first_separator()?;
        value.serialize(&mut *self.serializer)
    }

    // Close the sequence.
    fn end(self) -> Result<()> {
        write_array_end(&mut self.serializer.writer)?;
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a, W: std::io::Write> ser::SerializeTuple for ArraySerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

// Same thing but for tuple structs.
impl<'a, W: std::io::Write> ser::SerializeTupleStruct for ArraySerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: std::io::Write> ser::SerializeTupleVariant for ArraySerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a, W: std::io::Write> ser::SerializeMap for ArraySerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    // For _maps_, keyless JSON preserves the keys, similar to the output of `Object.entries({})`
    // in JavaScript, to handle objects with dynamic keys and values.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_or_skip_first_separator()?;
        key.serialize(&mut *self.serializer)?;
        Ok(())
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        write_object_separator(&mut self.serializer.writer)?;
        value.serialize(&mut *self.serializer)?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        write_object_end(&mut self.serializer.writer)?;
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a, W: std::io::Write> ser::SerializeStruct for ArraySerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.write_or_skip_first_separator()?;
        value.serialize(&mut *self.serializer)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a, W: std::io::Write> ser::SerializeStructVariant for ArraySerializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use serde::Serialize;

    use super::to_string;

    fn assert_ser<T: ?Sized + Serialize>(value: &T, expected: &str) {
        assert_eq!(to_string(value).unwrap(), expected);
    }

    #[test]
    fn test_bool() {
        // Booleans are encoded as 0/1 to give the shortest representation in bytes
        assert_ser(&true, "1");
        assert_ser(&false, "0");
    }

    #[test]
    fn test_struct() {
        #[derive(Serialize)]
        struct Test {
            int: u32,
            seq: Vec<&'static str>,
        }

        let test = Test {
            int: 5,
            seq: vec!["a", "b"],
        };
        assert_ser(&test, r#"[5,["a","b"]]"#);
    }

    #[test]
    fn test_newtype_variant() {
        #[derive(Serialize)]
        enum NewtypeTest {
            First(bool),
            Second(String),
        }

        let first = NewtypeTest::First(false);
        let second = NewtypeTest::Second("hello".into());
        assert_ser(&first, "[0,0]");
        assert_ser(&second, r#"[1,"hello"]"#);
    }

    #[test]
    fn test_enum() {
        #[derive(Serialize)]
        enum E {
            Unit,
            Newtype(u32),
            Tuple(u32, u32),
            Struct { a: u32 },
        }

        assert_ser(&E::Unit, "0");
        assert_ser(&E::Newtype(5), "[1,5]");
        assert_ser(&E::Tuple(1, 2), "[2,1,2]");
        assert_ser(&E::Struct { a: 1 }, "[3,1]");
    }

    #[test]
    fn test_map() {
        // Using a BTreeMap since it guarantees ordering based on the key, so the output will always
        // be in the same order.
        let mut map = BTreeMap::new();
        map.insert("first", "value1");
        map.insert("second", "value2");

        assert_ser(&map, r#"{"first":"value1","second":"value2"}"#);
    }
}
