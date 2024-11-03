import {
  AstNode,
  FullFormatJsNode,
  FormatJsNodeType,
  compressFormatJsToAst,
  isCompressedAst,
  AstNodeIndices,
  TagNode,
} from '@discord/intl-ast';

export class InternalIntlMessage {
  locale: string;
  ast: string | AstNode[];

  constructor(messageOrAst: AstNode[] | FullFormatJsNode[], locale: string) {
    this.locale = locale;
    this.ast = isCompressedAst(messageOrAst) ? messageOrAst : compressFormatJsToAst(messageOrAst);
  }

  /**
   * Return a stringified serialization of this message's AST, with no
   * formatting or values applied.
   */
  reserialize(): string {
    if (typeof this.ast === 'string') return this.ast;

    const result = { value: '' };
    serializeAst(this.ast, result);
    return result.value;
  }
}

// Accepting an object as the `result` parameter lets the same string get passed around and
// appended, rather than creating a bunch of intermediate strings.
function serializeAst(ast: AstNode[], result: { value: string }) {
  for (const node of ast) {
    if (typeof node === 'string') {
      result.value += node;
      continue;
    }

    switch (node[AstNodeIndices.Type]) {
      case FormatJsNodeType.Argument:
        result.value += '{' + node[AstNodeIndices.Value] + '}';
        break;
      case FormatJsNodeType.Date:
        result.value += '{' + node[AstNodeIndices.Value] + ', date';
        if (node[AstNodeIndices.Style] != null) {
          result.value += ', ' + node[AstNodeIndices.Style];
        }
        result.value += '}';
        break;
      case FormatJsNodeType.Time:
        result.value += '{' + node[AstNodeIndices.Value] + ', time';
        if (node[AstNodeIndices.Style] != null) {
          result.value += ', ' + node[AstNodeIndices.Style];
        }
        result.value += '}';
        break;
      case FormatJsNodeType.Number:
        result.value += '{' + node[AstNodeIndices.Value] + ', number';
        if (node[AstNodeIndices.Style] != null) {
          result.value += ', ' + node[AstNodeIndices.Style];
        }
        result.value += '}';
        break;
      case FormatJsNodeType.Plural: {
        const pluralType =
          node[AstNodeIndices.PluralType] == 'ordinal' ? 'selectordinal' : 'plural';
        result.value += '{' + node[AstNodeIndices.Value] + ', ' + pluralType + ',';
        if (node[AstNodeIndices.Offset]) {
          result.value += ' offset:' + node[AstNodeIndices.Offset];
        }
        for (const [name, arm] of Object.entries(node[AstNodeIndices.Options])) {
          result.value += ' ' + name + ' {';
          serializeAst(arm, result);
          result.value += '}';
        }
        result.value += '}';
        break;
      }
      case FormatJsNodeType.Pound:
        result.value += '#';
        break;
      case FormatJsNodeType.Select: {
        result.value += '{' + node[AstNodeIndices.Value] + ', select,';
        for (const [name, arm] of Object.entries(node[AstNodeIndices.Options])) {
          result.value += ' ' + name + ' {';
          serializeAst(arm, result);
          result.value += '}';
        }
        result.value += '}';
        break;
      }
      case FormatJsNodeType.Tag:
        serializeAstTag(node, result);
        break;
    }
  }
}

function serializeAstTag(node: TagNode, result: { value: string }) {
  switch (node[AstNodeIndices.Value]) {
    case '$b':
      result.value += '**';
      serializeAst(node[AstNodeIndices.Children], result);
      result.value += '**';
      break;
    case '$i':
      result.value += '*';
      serializeAst(node[AstNodeIndices.Children], result);
      result.value += '*';
      break;
    case '$code':
      result.value += '`';
      serializeAst(node[AstNodeIndices.Children], result);
      result.value += '`';
      break;
    case '$p':
      serializeAst(node[AstNodeIndices.Children], result);
      result.value += '\n\n';
      break;
    case '$link':
      // The target is the first child of the link. We don't have to care if it's a placeholder
      // or not, because the serialization will automatically remove the extra empty.
      const children = node[AstNodeIndices.Children], target = node[3]
      result.value += '[';
      serializeAst(children, result);
      result.value += '](';
      if (target != null) {
        serializeAst(target, result);
      }
      result.value += ')';
      break;
    default:
      // Any other tag name is a hook, which just adds the `$[` on a link tag.
      result.value += '$[';
      serializeAst(node[AstNodeIndices.Children], result);
      result.value += '](' + node[AstNodeIndices.Value] + ')';
      break;
  }
}
