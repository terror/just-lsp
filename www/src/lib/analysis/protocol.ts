import type { Diagnostic, Hover } from '../types';

export type AnalysisOperation =
  | { method: 'initialize' }
  | { method: 'analyze'; source: string }
  | {
      method: 'hover';
      source: string;
      position: { line: number; character: number };
    };

export type AnalysisRequest = AnalysisOperation & { id: number };

export type AnalysisResult =
  | { method: 'initialize'; result: undefined }
  | { method: 'analyze'; result: Diagnostic[] }
  | { method: 'hover'; result: Hover | undefined };

export type AnalysisResponse = { id: number } & (
  AnalysisResult | { error: string }
);
