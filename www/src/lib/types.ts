import { Extension } from '@codemirror/state';

export type Language =
  | 'cpp'
  | 'css'
  | 'go'
  | 'html'
  | 'java'
  | 'javascript'
  | 'json'
  | 'php'
  | 'python'
  | 'rust';

export interface LanguageConfig {
  name: Language;
  displayName: string;
  wasmPath: string;
  sampleCode: string;
  extension: Extension;
}

export type Position = {
  start: number;
  end: number;
};

export interface SyntaxNode {
  type: string;
  text: string;
  startPosition: { row: number; column: number };
  endPosition: { row: number; column: number };
  childCount: number;
  children: SyntaxNode[];
}

export interface TreeNode {
  text: string;
  node: SyntaxNode;
  level: number;
  type: string;
}
