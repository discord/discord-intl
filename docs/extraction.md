# Observational extraction reports

The report APIs use the existing interpreters. Definitions, file metadata,
message metadata, decoded text, raw native values, order, positions, and offsets
match the legacy results. Diagnostics observe data the interpreter already drops,
ignores, or decodes lossily; they do not remove results or introduce a validation
grammar. `complete` is false when observations exist, while those legacy results
remain available. Consumers choose how to handle an incomplete report.

## JavaScript

`JsMessageSource::extract_definitions_with_diagnostics(file, content)` returns
`Result<ExtractionReport, SourceParseError>`. The report contains the same `meta`
and `definitions` as `extract_definitions`, plus diagnostics and completeness.
Fatal syntax errors retain structured locations and recoverable parser errors
collected before failure. Recoverable errors do not suppress extracted values.

The marker import is accepted. Static template message values, import matching,
alias behavior, extra call arguments, spread tags, and source-order file metadata
retain their existing interpretation. Static template metadata remains ignored
where the legacy interpreter ignores it; it is not newly accepted or rejected.
Failed definitions and unsupported metadata generate observations alongside the
unchanged results. Lone surrogates still become U+FFFD in legacy JS decoding;
the report records `InvalidUnicode` and preserves that decoded result.

The optional escape crash patch bounds-checks slices that previously panicked on
short Unicode escapes. It deliberately retains existing offset calculations,
including their quirks, for previously successful inputs. It does not correct
hex-range classification, Unicode widths, or template offsets. It can be omitted
if preserving the crashing behavior is required.

## Flat JSON

`TranslationsJsonParser::with_diagnostics(content)` uses the same scanner, cursor
advancement, key handling, decoder rules, and positions as the legacy iterator.
After iteration, `diagnostics()` and `is_complete()` expose failures or discarded
trailing input. Existing permissive escape handling is preserved, including
unknown escapes such as `\q`. No JSON validation grammar or alternate parser
has been introduced. Report mode guards the decoder's unchecked Unicode slices
so failures can become diagnostics; the legacy decoder is unchanged.

`JsonMessageSource::extract_translations_with_diagnostics(file, content)` maps
these results into `ExtractionReport<RawMessageTranslation>`. Existing byte-based
JSON columns, line tracking, source-offset calculations, duplicates, and stopping
behavior are preserved. JSON has no definition metadata or import semantics.

## Shared API

Core exposes `SourceDiagnostic`, `DiagnosticCode`, `SourceParseError`,
`ExtractionReport<T>`, and `SourceOffsetList::as_slice()`. Diagnostic codes are
`ParseError`, `InvalidUnicode`, `IgnoredMetadata`, and `UnsupportedForm`. Diagnostics
carry a file, one-based line, zero-based UTF-16 column, half-open byte span, and
message. Message positions retain the units supplied by their source interpreter.

`public::extract_file_content_with_diagnostics(file_path, content)` dispatches
JS and JSON/JSONA sources. `IntlMessagesDatabase` exposes the corresponding
`extractFileContentWithDiagnostics(filePath, content)` N-API method. Reports
contain file metadata, messages, diagnostics, and completeness; message records
contain key, text, metadata, position, and offsets. Fatal JS parsing returns no
payload and retains its diagnostics. TypeScript declarations are generated from
the bindings.

There is no spelling store, spelling getter, occurrence history, owned rejection
API, or new insertion API. Existing native raw fields remain unchanged for
compatibility; portable reports omit raw spelling. Core insertion and duplicate
policies are untouched. The allocator feature remains default on and can be
disabled by Rust hosts without changing existing feature names.
