import { EditorState } from '@codemirror/state';
import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';
import { Language, Parser, Tree } from 'web-tree-sitter';

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export const parse = ({
  parser,
  language,
  code,
}: {
  parser: Parser;
  language: Language;
  code: string;
}): Tree | null => {
  parser.setLanguage(language);
  return parser.parse(code);
};

export const positionToOffset = (
  position: { row: number; column: number },
  doc: EditorState['doc']
): number | null => {
  const lineNumber = position.row + 1;

  if (lineNumber > doc.lines) {
    return null;
  }

  return (
    doc.line(lineNumber).from +
    Math.min(position.column, doc.line(lineNumber).length)
  );
};
