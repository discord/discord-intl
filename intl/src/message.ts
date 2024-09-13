import { FormatJsNode } from './keyless-json';

export class InternalIntlMessage {
  locale: string;
  ast: FormatJsNode[];

  constructor(messageOrAst: FormatJsNode[], locale: string) {
    this.locale = locale;
    this.ast = messageOrAst;
  }
}
