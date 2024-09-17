import { FormatJsNode, FormatJsNodeType, FormatJsTag } from './keyless-json';

export class InternalIntlMessage {
  locale: string;
  ast: FormatJsNode[];

  constructor(messageOrAst: FormatJsNode[], locale: string) {
    this.locale = locale;
    this.ast = messageOrAst;
  }

  /**
   * Return a stringified serialization of this message's AST, with no
   * formatting or values applied.
   */
  reserialize(): string {
    const value = '';
    serializeFormatJsAst(this.ast, { value });
    return value;
  }
}

// Accepting an object as the `result` parameter lets the same string get passed around and
// appended, rather than creating a bunch of intermediate strings.
function serializeFormatJsAst(ast: FormatJsNode[], result: { value: string }) {
  for (const node of ast) {
    switch (node.type) {
      case FormatJsNodeType.Literal:
        result.value += node.value;
        return;
      case FormatJsNodeType.Argument:
        // Empties are an artifact of our parsing strategy, not necessary here.
        if (result.value === '$_') return;
        result.value += '{' + node.value + '}';
        return;
      case FormatJsNodeType.Date:
        result.value += '{' + node.value + ', date';
        if (node.style != null) {
          result.value += ', ' + node.style;
        }
        result.value += '}';
        return;
      case FormatJsNodeType.Time:
        result.value += '{' + node.value + ', time';
        if (node.style != null) {
          result.value += ', ' + node.style;
        }
        result.value += '}';
        return;
      case FormatJsNodeType.Number:
        result.value += '{' + node.value + ', number';
        if (node.style != null) {
          result.value += ', ' + node.style;
        }
        result.value += '}';
        return;
      case FormatJsNodeType.Plural: {
        const pluralType = node.pluralType == 'ordinal' ? 'selectordinal' : 'plural';
        result.value += '{' + node.value + ', ' + pluralType + ', ';
        if (node.offset) {
          result.value += 'offset:' + node.offset;
        }
        for (const [name, arm] of Object.entries(node.options)) {
          result.value += ' ' + name + ' {';
          serializeFormatJsAst(arm.value, result);
          result.value += '}';
        }
        return;
      }
      case FormatJsNodeType.Pound:
        result.value += '#';
        return;
      case FormatJsNodeType.Select: {
        result.value += '{' + node.value + ', select, ';
        for (const [name, arm] of Object.entries(node.options)) {
          result.value += ' ' + name + ' {';
          serializeFormatJsAst(arm.value, result);
          result.value += '}';
        }
        return;
      }
      case FormatJsNodeType.Tag:
        serializeFormatJsTag(node, result);
        return;
    }
  }
}

function serializeFormatJsTag(node: FormatJsTag, result: { value: string }) {
  switch (node.value) {
    case '$b':
      result.value += '**';
      serializeFormatJsAst(node.children, result);
      result.value += '**';
      return;
    case '$i':
      result.value += '*';
      serializeFormatJsAst(node.children, result);
      result.value += '*';
      return;
    case '$code':
      result.value += '`';
      serializeFormatJsAst(node.children, result);
      result.value += '`';
      return;
    case '$p':
      serializeFormatJsAst(node.children, result);
      result.value += '\n\n';
      return;
    case '$link':
      // The target is the first child of the link. We don't have to care if it's a placeholder
      // or not, because the serialization will automatically remove the extra empty.
      const [target, ...children] = node.children;
      result.value += '[';
      serializeFormatJsAst(children, result);
      result.value += '](';
      if (target != null) {
        serializeFormatJsAst([target], result);
      }
      result.value += ')';
      return;
    default:
      // Any other tag name is a hook, which just adds the `$[` on a link tag.
      result.value += '$[';
      serializeFormatJsAst(node.children, result);
      result.value += '](' + node.value + ')';
      return;
  }
}
