# Keyless JSON

Keyless JSON is a minified, array-based serialization format that allows well-known structures to be efficiently
represented without any duplication of key names. Keyless JSON is similar to MessagePack with the Record extension,
using structures known ahead of time to greatly reduce the size of the serialized data.

Deserializing Keyless JSON requires knowing the expected structure and _will fail_ if the data does not match the
expected structure.

For example, an AST for a localized message might look like this when serialized to plain JSON:

```text
Hello, {username}. You have {messageCount, plural, one {# new message.} other {# new messages.}}
```

```json
[
  {
    "type": 0,
    "value": "Hello, "
  },
  {
    "type": 1,
    "value": "username"
  },
  {
    "type": 0,
    "value": ". You have "
  },
  {
    "type": 6,
    "value": "messageCount",
    "options": {
      "one": {
        "value": [
          {
            "type": 7
          },
          {
            "type": 0,
            "value": " new message."
          }
        ]
      },
      "other": {
        "value": [
          {
            "type": 7
          },
          {
            "type": 0,
            "value": " new messages."
          }
        ]
      }
    }
  }
]
```

When serialized with keyless JSON, the repeated `type` and `value` keys can be left out thanks to the well-known
structure of each node in this tree (each node has a `type` and `value` in the same order):

```json
[
  [0, "Hello, "],
  [1, "username"],
  [0, ". You have"],
  [
    6,
    "messageCount",
    {
      "one": {
        "value": [[7], [0, "new message."]]
      },
      "other": {
        "value": [[7], [0, "new messages."]]
      }
    }
  ]
]
```

These examples are not minified fully, but showcase just how much repetition can be omitted by pre-defining the keys.
