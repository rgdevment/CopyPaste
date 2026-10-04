import MarkdownIt from "markdown-it";

const md = new MarkdownIt({ html: false, linkify: true });

export const composed = (text: string): string =>
  md.render(text.replace(/<!--[\s\S]*?-->\s*/g, ""));
