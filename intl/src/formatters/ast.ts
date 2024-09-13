import {
  AnyIntlMessage,
  FormatterImplementation,
  FunctionTypes,
  type RichTextFormattingMap,
  RichTextTagNames,
} from '../types';
import { IntlManager } from '../intl-manager';
import { FormatBuilder } from '../format';

/**
 * Types for formatting functions when calling `formatToParts`, ensuring the
 * functions yield value AST nodes.
 */
export type AstFunctionTypes = FunctionTypes<RichTextNode>;

// This structure aims to match `simple-markdown`'s AST, but that type is very
// loosely defined as just "type" and "arbitrary map of content". This is more
// explicit and type-safe.
export enum RichTextNodeType {
  Text = 'text',
  Strong = 'strong',
  Emphasis = 'emphasis',
  Code = 'code',
  Link = 'link',
  Paragraph = 'paragraph',
  /**
   * A special extension for when the value of a placeholder is an object.
   * Rather than casting to a string here, the object is preserved as-is for a
   * future formatter to render as appropriate (i.e., using an object as a
   * value when formatting to an AST should keep that object structure so that
   * the final AST is accurate for what the user expects, but when rendering to
   * a plain string, the final formatter can just cast that object to a string
   * using its `toString` method or some other mechanism).
   */
  Object = 'object',
}

const RICH_TEXT_TAG_TYPES: Record<RichTextTagNames, RichTextNodeType> = {
  $_: RichTextNodeType.Text,
  $b: RichTextNodeType.Strong,
  $i: RichTextNodeType.Emphasis,
  $code: RichTextNodeType.Code,
  $link: RichTextNodeType.Link,
  $p: RichTextNodeType.Paragraph,
};

interface RichTextNodeBase<ContentType> {
  type: RichTextNodeType;
  content: ContentType;
}
interface RichTextTextNode extends RichTextNodeBase<string> {
  type: RichTextNodeType.Text;
}
interface RichTextStrongNode extends RichTextNodeBase<RichTextNode[]> {
  type: RichTextNodeType.Strong;
}
interface RichTextEmphasisNode extends RichTextNodeBase<RichTextNode[]> {
  type: RichTextNodeType.Emphasis;
}
interface RichTextCodeNode extends RichTextNodeBase<RichTextNode[]> {
  type: RichTextNodeType.Code;
}
interface RichTextParagraphNode extends RichTextNodeBase<RichTextNode[]> {
  type: RichTextNodeType.Paragraph;
}
interface RichTextLinkNode extends RichTextNodeBase<RichTextNode[]> {
  type: RichTextNodeType.Link;
  target: string;
}
interface RichTextObjectNode extends RichTextNodeBase<object> {
  type: RichTextNodeType.Object;
}

export type RichTextNode =
  | RichTextTextNode
  | RichTextStrongNode
  | RichTextEmphasisNode
  | RichTextCodeNode
  | RichTextParagraphNode
  | RichTextLinkNode
  | RichTextObjectNode;

const AST_RICH_TEXT_ELEMENTS: RichTextFormattingMap<AstFunctionTypes['hook']> = {
  $_: () => ({ type: RichTextNodeType.Text, content: '' }),
  $b: (content) => ({ type: RichTextNodeType.Strong, content }),
  $code: (content) => ({ type: RichTextNodeType.Code, content }),
  $i: (content) => ({ type: RichTextNodeType.Emphasis, content }),
  $link: ([target, ...content]) => ({
    type: RichTextNodeType.Link,
    target: target.content as string,
    content,
  }),
  $p: (content) => ({ type: RichTextNodeType.Paragraph, content }),
};

class AstBuilder extends FormatBuilder<RichTextNode> {
  result: RichTextNode[] = [];

  pushRichTextTag(tag: RichTextTagNames, children: RichTextNode[]) {
    if (!(tag in AST_RICH_TEXT_ELEMENTS)) {
      throw `${tag} is not a known rich text formatting tag`;
    }
    const result = AST_RICH_TEXT_ELEMENTS[tag](children);
    if (Array.isArray(result)) {
      this.result.push(...result);
    } else {
      this.result.push(result);
    }
  }

  pushLiteralText(text: string) {
    const last = this.result[this.result.length - 1];
    if (last.type === RichTextNodeType.Text) {
      last.content += text;
    } else {
      this.result.push({ type: RichTextNodeType.Text, content: text });
    }
  }

  pushObject(value: object) {
    this.result.push({ type: RichTextNodeType.Object, content: value });
  }

  finish(): RichTextNode[] {
    return this.result;
  }
}

/**
 * Format the given message in the current locale with the provided values.
 * The returned values is _always_ an Array of parts, even if the message is
 * a simple string value.
 *
 * This function is the basis of how messages are normally formatted, and can
 * be used anywhere. However, it is not reactive and only functions on the
 * data that is currently loaded and known. For a reactive function that
 * automatically updates when the locale changes or when new data is loaded,
 * use `format`, which will wrap the formatting in a React component that
 * subscribes to the current locale and state of loaded messages.
 */
export function formatToAst(
  this: IntlManager,
  message: AnyIntlMessage,
  values: object,
): RichTextNode[] {
  if (typeof message === 'string') return [{ type: RichTextNodeType.Text, content: message }];

  return this.bindFormatValues(AstBuilder, message, values);
}

export const astFormatter: FormatterImplementation<AstFunctionTypes, RichTextNode[]> = {
  format: formatToAst,
  builder: AstBuilder,
};
