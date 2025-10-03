import { EditorState } from '@codemirror/state';
import { describe, expect, it, mock } from 'bun:test';
import { type Tree } from 'web-tree-sitter';

import {
  cn,
  createNodePositionMap,
  createNodeToParentMap,
  formatTree,
  getAncestors,
  getVisibleNodes,
  parse,
  positionToOffset,
  processTree,
} from './utils';

const mockTreeSitterNode = (
  type: string,
  startRow: number,
  startCol: number,
  endRow: number,
  endCol: number,
  children: any[] = []
) => ({
  type,
  text: `Sample ${type}`,
  startPosition: { row: startRow, column: startCol },
  endPosition: { row: endRow, column: endCol },
  childCount: children.length,
  children,
});

describe('cn utility', () => {
  it('merges class names correctly', () => {
    expect(cn('foo', 'bar')).toBe('foo bar');
    expect(cn('foo', { bar: true })).toBe('foo bar');
    expect(cn('foo', { bar: false })).toBe('foo');
    expect(cn('foo', ['bar', 'baz'])).toBe('foo bar baz');
  });
});

describe('positionToOffset', () => {
  it('returns null when line number exceeds document lines', () => {
    const doc = EditorState.create({ doc: 'line1\nline2' }).doc;
    expect(positionToOffset({ row: 2, column: 0 }, doc)).toBeNull();
  });

  it('converts position to offset correctly', () => {
    const doc = EditorState.create({ doc: 'line1\nline2\nline3' }).doc;

    expect(positionToOffset({ row: 0, column: 0 }, doc)).toBe(0);
    expect(positionToOffset({ row: 0, column: 3 }, doc)).toBe(3);
    expect(positionToOffset({ row: 1, column: 0 }, doc)).toBe(6);
    expect(positionToOffset({ row: 1, column: 2 }, doc)).toBe(8);
    expect(positionToOffset({ row: 0, column: 100 }, doc)).toBe(5);
  });
});

describe('formatTree', () => {
  it('formats a tree with no children', () => {
    const node = mockTreeSitterNode('identifier', 0, 0, 0, 5, []);
    const result = formatTree(node);

    expect(result.length).toBe(1);
    expect(result[0]).toEqual({
      text: 'identifier [0,0]',
      node,
      level: 0,
      type: 'identifier',
    });
  });

  it('formats a tree with nested children', () => {
    const childNode1 = mockTreeSitterNode('string', 0, 2, 0, 5, []);
    const childNode2 = mockTreeSitterNode('number', 0, 6, 0, 8, []);
    const grandChildNode = mockTreeSitterNode('digit', 0, 6, 0, 7, []);

    childNode2.children = [grandChildNode];
    childNode2.childCount = 1;

    const rootNode = mockTreeSitterNode('expression', 0, 0, 0, 10, [
      childNode1,
      childNode2,
    ]);

    const result = formatTree(rootNode);

    expect(result.length).toBe(4);
    expect(result[0].type).toBe('expression');
    expect(result[0].level).toBe(0);
    expect(result[1].type).toBe('string');
    expect(result[1].level).toBe(1);
    expect(result[2].type).toBe('number');
    expect(result[2].level).toBe(1);
    expect(result[3].type).toBe('digit');
    expect(result[3].level).toBe(2);
  });
});

describe('createNodeToParentMap', () => {
  it('creates correct parent-child relationships', () => {
    const rootNode = mockTreeSitterNode('program', 0, 0, 5, 10, []);
    const childNode = mockTreeSitterNode('function', 1, 0, 3, 10, []);
    const grandchildNode = mockTreeSitterNode('identifier', 1, 1, 1, 5, []);

    const formattedTree = [
      { node: rootNode, level: 0, text: '', type: 'program' },
      { node: childNode, level: 1, text: '', type: 'function' },
      { node: grandchildNode, level: 2, text: '', type: 'identifier' },
    ];

    const parentMap = createNodeToParentMap(formattedTree);

    expect(parentMap.size).toBe(2);
    expect(parentMap.get(childNode)).toBe(rootNode);
    expect(parentMap.get(grandchildNode)).toBe(childNode);
  });

  it('handles multiple children at the same level', () => {
    const rootNode = mockTreeSitterNode('program', 0, 0, 5, 10, []);
    const child1 = mockTreeSitterNode('function', 1, 0, 2, 5, []);
    const child2 = mockTreeSitterNode('variable', 3, 0, 4, 5, []);

    const formattedTree = [
      { node: rootNode, level: 0, text: '', type: 'program' },
      { node: child1, level: 1, text: '', type: 'function' },
      { node: child2, level: 1, text: '', type: 'variable' },
    ];

    const parentMap = createNodeToParentMap(formattedTree);

    expect(parentMap.size).toBe(2);
    expect(parentMap.get(child1)).toBe(rootNode);
    expect(parentMap.get(child2)).toBe(rootNode);
  });
});

describe('createNodePositionMap', () => {
  it('maps nodes to their document positions', () => {
    const doc = EditorState.create({
      doc: 'function test() {\n  return true;\n}',
    }).doc;

    const rootNode = mockTreeSitterNode('program', 0, 0, 2, 1, []);
    const funcNode = mockTreeSitterNode('function', 0, 0, 2, 1, []);

    const formattedTree = [
      { node: rootNode, level: 0, text: '', type: 'program' },
      { node: funcNode, level: 1, text: '', type: 'function' },
    ];

    const positionMap = createNodePositionMap(formattedTree, doc);

    expect(positionMap.size).toBe(2);
    expect(positionMap.get(rootNode)).toEqual({ start: 0, end: 34 });
    expect(positionMap.get(funcNode)).toEqual({ start: 0, end: 34 });
  });

  it('skips nodes with invalid positions', () => {
    const doc = EditorState.create({ doc: 'short text' }).doc;

    const rootNode = mockTreeSitterNode('program', 0, 0, 0, 10, []);
    const invalidNode = mockTreeSitterNode('invalid', 5, 0, 6, 5, []);

    const formattedTree = [
      { node: rootNode, level: 0, text: '', type: 'program' },
      { node: invalidNode, level: 1, text: '', type: 'invalid' },
    ];

    const positionMap = createNodePositionMap(formattedTree, doc);

    expect(positionMap.size).toBe(1);
    expect(positionMap.has(rootNode)).toBe(true);
    expect(positionMap.has(invalidNode)).toBe(false);
  });
});

describe('getAncestors', () => {
  it('returns all ancestors in order', () => {
    const rootNode = mockTreeSitterNode('program', 0, 0, 5, 0, []);
    const level1Node = mockTreeSitterNode('block', 1, 0, 4, 0, []);
    const level2Node = mockTreeSitterNode('statement', 2, 0, 3, 0, []);
    const leafNode = mockTreeSitterNode('expression', 2, 2, 2, 10, []);

    const nodeToParentMap = new Map([
      [level1Node, rootNode],
      [level2Node, level1Node],
      [leafNode, level2Node],
    ]);

    const ancestors = getAncestors(leafNode, nodeToParentMap);

    expect(ancestors.length).toBe(3);
    expect(ancestors[0]).toBe(level2Node);
    expect(ancestors[1]).toBe(level1Node);
    expect(ancestors[2]).toBe(rootNode);
  });

  it('returns empty array for root node', () => {
    const rootNode = mockTreeSitterNode('program', 0, 0, 5, 0, []);
    const childNode = mockTreeSitterNode('block', 1, 0, 4, 0, []);

    const nodeToParentMap = new Map([[childNode, rootNode]]);

    const ancestors = getAncestors(rootNode, nodeToParentMap);

    expect(ancestors.length).toBe(0);
  });
});

describe('getVisibleNodes', () => {
  it('returns only visible nodes based on expanded state', () => {
    const rootNode = mockTreeSitterNode('program', 0, 0, 5, 0, []);
    const expandedNode = mockTreeSitterNode('block', 1, 0, 4, 0, []);
    const collapsedNode = mockTreeSitterNode('function', 1, 5, 3, 0, []);
    const childOfExpanded = mockTreeSitterNode('statement', 2, 0, 2, 10, []);
    const childOfCollapsed = mockTreeSitterNode('parameter', 2, 5, 2, 8, []);

    const formattedTree = [
      { node: rootNode, level: 0, text: '', type: 'program' },
      { node: expandedNode, level: 1, text: '', type: 'block' },
      { node: childOfExpanded, level: 2, text: '', type: 'statement' },
      { node: collapsedNode, level: 1, text: '', type: 'function' },
      { node: childOfCollapsed, level: 2, text: '', type: 'parameter' },
    ];

    const expandedNodes = new Set([rootNode, expandedNode]);

    const visibleNodes = getVisibleNodes(formattedTree, expandedNodes);

    expect(visibleNodes.length).toBe(4);
    expect(visibleNodes[0].node).toBe(rootNode);
    expect(visibleNodes[1].node).toBe(expandedNode);
    expect(visibleNodes[2].node).toBe(childOfExpanded);
    expect(visibleNodes[3].node).toBe(collapsedNode);
  });

  it('returns empty array for empty tree', () => {
    const result = getVisibleNodes([], new Set());
    expect(result).toEqual([]);
  });
});

describe('parse', () => {
  it('sets language and calls parse', () => {
    const mockParser = {
      setLanguage: mock(() => {}),
      parse: mock(() => ({ rootNode: {} }) as unknown as Tree),
    };

    const mockLanguage = { name: 'javascript' };
    const code = 'const x = 1;';

    const result = parse({
      parser: mockParser as any,
      language: mockLanguage as any,
      code,
    });

    expect(mockParser.setLanguage).toHaveBeenCalledWith(mockLanguage);
    expect(mockParser.parse).toHaveBeenCalledWith(code);

    expect(result).toBeDefined();
  });
});

describe('processTree', () => {
  it('processes a tree into required data structures', () => {
    const rootNode = mockTreeSitterNode('program', 0, 0, 1, 0, []);
    const mockTree = { rootNode } as unknown as Tree;
    const doc = EditorState.create({ doc: 'test' }).doc;

    const result = processTree(mockTree, doc);

    expect(result).toHaveProperty('formattedTree');
    expect(result).toHaveProperty('nodePositionMap');
    expect(result).toHaveProperty('allNodes');
    expect(Array.isArray(result.formattedTree)).toBe(true);
    expect(result.nodePositionMap instanceof Map).toBe(true);
    expect(result.allNodes instanceof Set).toBe(true);
  });
});
