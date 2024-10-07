"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.FORMAT_JS_POUND = exports.FormatJsNodeType = void 0;
exports.hydrateFormatJsAst = hydrateFormatJsAst;
exports.hydrateMessages = hydrateMessages;
var FormatJsNodeType;
(function (FormatJsNodeType) {
    FormatJsNodeType[FormatJsNodeType["Literal"] = 0] = "Literal";
    FormatJsNodeType[FormatJsNodeType["Argument"] = 1] = "Argument";
    FormatJsNodeType[FormatJsNodeType["Number"] = 2] = "Number";
    FormatJsNodeType[FormatJsNodeType["Date"] = 3] = "Date";
    FormatJsNodeType[FormatJsNodeType["Time"] = 4] = "Time";
    FormatJsNodeType[FormatJsNodeType["Select"] = 5] = "Select";
    FormatJsNodeType[FormatJsNodeType["Plural"] = 6] = "Plural";
    FormatJsNodeType[FormatJsNodeType["Pound"] = 7] = "Pound";
    FormatJsNodeType[FormatJsNodeType["Tag"] = 8] = "Tag";
})(FormatJsNodeType || (exports.FormatJsNodeType = FormatJsNodeType = {}));
// Using a static object here ensures there's only one allocation for it, which
// will likely appear in many, many messages. Freezing ensures that the shared
// object can't be accidentally modified if a consumer tries to transform the
// AST.
exports.FORMAT_JS_POUND = Object.freeze({ type: 7 });
function hydrateArray(elements) {
    for (var i = 0; i < elements.length; i++) {
        elements[i] = hydrateFormatJsAst(elements[i]);
    }
}
function hydratePlural(keyless) {
    var type = keyless[0], value = keyless[1], options = keyless[2], offset = keyless[3], pluralType = keyless[4];
    // This tries to be efficient about updating the value for each option
    // with a parsed version, reusing the same receiving object, and even
    // the inner key within it, just replacing the end value with the
    // parsed version.
    // This saves multiple allocations compared to building a new object
    // from scratch, either for the whole options object or for each value.
    for (var key in options) {
        hydrateArray(options[key].value);
    }
    // `pluralType` is technically only valid on `Plural` nodes, even
    // though the structure is identical to `Select`.
    if (type === FormatJsNodeType.Plural) {
        return { type: type, value: value, options: options, offset: offset, pluralType: pluralType };
    }
    else {
        return { type: type, value: value, options: options, offset: offset };
    }
}
function hydrateSingle(keyless) {
    var type = keyless[0];
    switch (type) {
        case FormatJsNodeType.Literal:
        case FormatJsNodeType.Argument:
            return { type: type, value: keyless[1] };
        case FormatJsNodeType.Number:
        case FormatJsNodeType.Date:
        case FormatJsNodeType.Time:
            return { type: type, value: keyless[1], style: keyless[2] };
        case FormatJsNodeType.Select:
        case FormatJsNodeType.Plural:
            return hydratePlural(keyless);
        case FormatJsNodeType.Pound:
            return exports.FORMAT_JS_POUND;
        case FormatJsNodeType.Tag: {
            var type_1 = keyless[0], value = keyless[1], children = keyless[2];
            hydrateArray(children);
            return { type: type_1, value: value, children: children };
        }
        default:
            throw new Error("FormatJS keyless JSON encountered an unknown type: ".concat(type));
    }
}
function hydrateFormatJsAst(keyless) {
    // If the first element of the array is itself an array, then we have a list
    // of elements to parse rather than a single object.
    if (Array.isArray(keyless[0])) {
        hydrateArray(keyless);
        return keyless;
    }
    else if (keyless.length === 0) {
        // Some entries can be empty arrays, like an empty set of children, and those can
        // be preserved.
        return keyless;
    }
    return hydrateSingle(keyless);
}
function hydrateMessages(messages) {
    for (var key in messages) {
        messages[key] = hydrateFormatJsAst(messages[key]);
    }
    return messages;
}
