export interface SyntaxNode {
  id: number;
  type: string;
  text: string;
  startPosition: { row: number; column: number };
  endPosition: { row: number; column: number };
  childCount: number;
  children: SyntaxNode[];
}
