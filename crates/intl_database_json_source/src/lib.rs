use intl_database_core::{
    key_symbol, FilePosition, KeySymbol, MessageSourceResult, MessageTranslationSource,
    RawMessageTranslation, SourceOffsetList,
};
use intl_flat_json_parser::{parse_flat_translation_json, JsonMessage};
use std::path::Path;

pub struct JsonMessageSource;

fn make_source_offsets(message: &JsonMessage) -> SourceOffsetList {
    if message.raw == message.value {
        return SourceOffsetList::default();
    }
    let mut list: Vec<(u32, u32)> = vec![];

    let bytes = message.raw.as_bytes();
    let mut total_offset = 0;
    let mut last_checked_byte = 0;
    for idx in memchr::memchr_iter(b'\\', bytes) {
        let Some(byte) = bytes.get(idx + 1) else {
            continue;
        };
        if idx < last_checked_byte {
            continue;
        }

        // Section 9 of https://ecma-international.org/wp-content/uploads/ECMA-404.pdf defines
        // valid escape sequences.
        let added_offset = match byte {
            b'b' | b'f' | b'n' | b'r' | b't' | b'\\' | b'/' | b'"' => 1,
            b'u' => 4,
            _ => 0,
        };

        last_checked_byte = idx + added_offset;
        if added_offset > 0 {
            let this_offset = idx - total_offset;
            total_offset += added_offset;
            list.push((this_offset as u32, total_offset as u32));
        }
    }
    SourceOffsetList::new(list)
}

impl MessageTranslationSource for JsonMessageSource {
    fn get_locale_from_file_name(&self, file_name: &str) -> KeySymbol {
        Path::new(file_name)
            .file_name()
            .and_then(|p| p.to_str())
            .map(|p| p.split_once(".").map_or(p, |(name, _ext)| name))
            .unwrap_or("en-US")
            .into()
    }

    fn extract_translations(
        self,
        file_name: KeySymbol,
        content: &str,
    ) -> MessageSourceResult<impl Iterator<Item = RawMessageTranslation>> {
        let iter = parse_flat_translation_json(&content);
        Ok(iter.map(move |item| {
            let source_offsets = make_source_offsets(&item);
            RawMessageTranslation::new(
                key_symbol(&item.key),
                FilePosition::new(file_name, item.position.line, item.position.col),
                item.value,
                source_offsets,
            )
        }))
    }
}

#[test]
fn test_locale_from_file_name() {
    assert_eq!(
        key_symbol("en-US"),
        JsonMessageSource.get_locale_from_file_name("foo/bar/baz/en-US.messages.jsona")
    );
    assert_eq!(
        key_symbol("fr-FR"),
        JsonMessageSource.get_locale_from_file_name("foo/bar/baz/fr-FR.messages.jsona")
    );
    assert_eq!(
        key_symbol("notareal__locale#1"),
        JsonMessageSource.get_locale_from_file_name("notareal__locale#1.messages.jsona")
    );

    assert_eq!(
        key_symbol("da"),
        JsonMessageSource.get_locale_from_file_name("da.messages.jsona")
    );
    assert_eq!(
        key_symbol("cz"),
        JsonMessageSource.get_locale_from_file_name("foo/bar/cz")
    );
}
