# intl_flat_json_parser

A Rust-based JSON parser with location information for flat objects, used by `@discord/intl-message-database` to provide more accurate diagnostics for values parsed out of JSON files.

The parser is specifically built to only understand a flat, well-formed object structure and has minimal error recovery, but fully supports Unicode and other JSON string syntax like escapes.

## Install

```
pnpm add @discord/intl-flat-json-parser
```

## Usage

```typescript
import { parseJson, parseJsonFile } from '@discord/intl-flat-json-parser';

// Parse a string as flat JSON
const jsonContent = `{
  "MESSAGE_ONE": "Hello, this is the first message",
  "MESSAGE_TWO": "Another message!"
}`;

const messages = parseJson(jsonContent);
console.log(messages[0]);
//=> {
//   key: "MESSAGE_ONE",
//   value: "Hello, this is the first message",
//   position: {
//     line: 2,
//     col: 19
//   }
// }

// Or pass a file path to get parsed directly from the file system
const directMessages = parseJsonFile('some/file/path.json');
```
