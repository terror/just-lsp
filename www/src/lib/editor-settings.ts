export const FONT_SIZES = [12, 14, 16, 18] as const;
export const TAB_SIZES = [2, 4, 8] as const;

export interface EditorSettings {
  fontSize: number;
  keybindings: 'default' | 'vim';
  lineNumbers: boolean;
  lineWrapping: boolean;
  tabSize: number;
}

export const defaultSettings = {
  fontSize: 14,
  keybindings: 'default',
  lineNumbers: true,
  lineWrapping: true,
  tabSize: 2,
} as const satisfies EditorSettings;
