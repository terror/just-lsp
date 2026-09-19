import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import {
  EditorSettings,
  FONT_SIZES,
  TAB_SIZES,
  defaultSettings,
} from '@/lib/editor-settings';
import { RotateCcw, Settings } from 'lucide-react';
import { ReactNode, useId } from 'react';

const SettingSection = ({
  children,
  title,
}: {
  children: ReactNode;
  title: string;
}) => (
  <section className='grid gap-3'>
    <h3 className='text-muted-foreground text-xs font-medium'>{title}</h3>
    <div className='grid divide-y rounded-md border'>{children}</div>
  </section>
);

const SettingRow = ({
  children,
  description,
  id,
  label,
}: {
  children: ReactNode;
  description: string;
  id: string;
  label: string;
}) => (
  <div className='grid gap-3 p-3 sm:grid-cols-[1fr_auto] sm:items-center sm:gap-6'>
    <div className='grid gap-1'>
      <Label htmlFor={id} className='cursor-pointer text-sm font-medium'>
        {label}
      </Label>
      <p id={`${id}-description`} className='text-muted-foreground text-xs'>
        {description}
      </p>
    </div>
    {children}
  </div>
);

interface EditorSettingsDialogProps {
  settings: EditorSettings;
  updateSettings: (settings: Partial<EditorSettings>) => void;
}

export const EditorSettingsDialog = ({
  settings,
  updateSettings,
}: EditorSettingsDialogProps) => {
  const id = useId();
  const hasChanges = Object.entries(defaultSettings).some(
    ([key, value]) => settings[key as keyof EditorSettings] !== value
  );

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button
          variant='ghost'
          size='icon'
          title='Settings'
          aria-label='Settings'
          className='h-7 w-7 cursor-pointer'
        >
          <Settings className='h-4 w-4' aria-hidden='true' />
        </Button>
      </DialogTrigger>
      <DialogContent className='max-h-[calc(100dvh-2rem)] gap-0 overflow-y-auto p-0 sm:max-w-[560px]'>
        <DialogHeader className='px-6 pt-6 pb-2'>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>
            Customize your editor experience with these settings.
          </DialogDescription>
        </DialogHeader>
        <div className='grid gap-5 px-6 py-5'>
          <SettingSection title='Display'>
            <SettingRow
              id={`${id}-line-numbers`}
              label='Line numbers'
              description='Show line numbers beside the editor.'
            >
              <Switch
                id={`${id}-line-numbers`}
                aria-describedby={`${id}-line-numbers-description`}
                checked={settings.lineNumbers}
                onCheckedChange={(checked) =>
                  updateSettings({ lineNumbers: checked })
                }
              />
            </SettingRow>
            <SettingRow
              id={`${id}-word-wrap`}
              label='Word wrap'
              description='Keep long lines within the editor pane.'
            >
              <Switch
                id={`${id}-word-wrap`}
                aria-describedby={`${id}-word-wrap-description`}
                checked={settings.lineWrapping}
                onCheckedChange={(checked) =>
                  updateSettings({ lineWrapping: checked })
                }
              />
            </SettingRow>
          </SettingSection>
          <SettingSection title='Editing'>
            <SettingRow
              id={`${id}-font-size`}
              label='Font size'
              description='Adjust the editor text size.'
            >
              <Select
                value={settings.fontSize.toString()}
                onValueChange={(value) =>
                  updateSettings({ fontSize: parseInt(value) })
                }
              >
                <SelectTrigger
                  id={`${id}-font-size`}
                  aria-describedby={`${id}-font-size-description`}
                  className='w-full sm:w-32'
                >
                  <SelectValue placeholder='Font size' />
                </SelectTrigger>
                <SelectContent>
                  {FONT_SIZES.map((size) => (
                    <SelectItem key={size} value={size.toString()}>
                      {size}px
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </SettingRow>
            <SettingRow
              id={`${id}-tab-size`}
              label='Tab size'
              description='Set the width of tab characters.'
            >
              <Select
                value={settings.tabSize.toString()}
                onValueChange={(value) =>
                  updateSettings({ tabSize: parseInt(value) })
                }
              >
                <SelectTrigger
                  id={`${id}-tab-size`}
                  aria-describedby={`${id}-tab-size-description`}
                  className='w-full sm:w-32'
                >
                  <SelectValue placeholder='Tab size' />
                </SelectTrigger>
                <SelectContent>
                  {TAB_SIZES.map((size) => (
                    <SelectItem key={size} value={size.toString()}>
                      {size} spaces
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </SettingRow>
            <SettingRow
              id={`${id}-keybindings`}
              label='Keybindings'
              description='Choose standard shortcuts or Vim bindings.'
            >
              <Select
                value={settings.keybindings}
                onValueChange={(value) =>
                  updateSettings({ keybindings: value as 'default' | 'vim' })
                }
              >
                <SelectTrigger
                  id={`${id}-keybindings`}
                  aria-describedby={`${id}-keybindings-description`}
                  className='w-full sm:w-32'
                >
                  <SelectValue placeholder='Default' />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='default'>Default</SelectItem>
                  <SelectItem value='vim'>Vim</SelectItem>
                </SelectContent>
              </Select>
            </SettingRow>
          </SettingSection>
        </div>
        <DialogFooter className='px-6 py-6'>
          <Button
            variant='outline'
            size='sm'
            onClick={() => updateSettings(defaultSettings)}
            disabled={!hasChanges}
          >
            <RotateCcw className='h-3.5 w-3.5' aria-hidden='true' />
            Reset
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};
