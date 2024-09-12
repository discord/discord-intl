import { FormatJsNode, FormatJsNodeType } from './keyless-json';

export class InternalIntlMessage {
  locale: string;
  message?: string;
  ast: FormatJsNode[];

  /**
   * A stripped-down representation of the message with no rich formatting
   * elements preserved. This will be lazily created the first time it is
   * requested (e.g., by calling `formatToPlainString`).
   */
  get plainAst(): FormatJsNode[] {
    return (this._plainAst = this._removeRichTags(this.ast));
  }
  _plainAst: FormatJsNode[];

  constructor(messageOrAst: string | FormatJsNode[], locale: string) {
    this.locale = locale;

    if (typeof messageOrAst === 'string') {
      this.message = messageOrAst;
    } else {
      this.ast = messageOrAst;
    }
  }

  /**
   * Returns the same element with any formatting tags removed. If the element
   * itself is a tag, the children will be hoisted and returned, otherwise the
   * element is returned as an array of itself.
   */
  private _removeRichTags(nodes: FormatJsNode[]): FormatJsNode[] {
    const result = new Array(nodes.length);
    for (const node of nodes) {
      if (node.type === FormatJsNodeType.Tag) {
        result.push(...this._removeRichTags(node.children));
      } else {
        result.push(node);
      }
    }
    return result;
  }
}
