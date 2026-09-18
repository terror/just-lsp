import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
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
import { EditorSettings, FONT_SIZES, TAB_SIZES } from '@/lib/editor-settings';
import { Settings } from 'lucide-react';
import { useId } from 'react';

interface EditorSettingsDialogProps {
  settings: EditorSettings;
  updateSettings: (settings: Partial<EditorSettings>) => void;
}

export const EditorSettingsDialog = ({
  settings,
  updateSettings,
}: EditorSettingsDialogProps) => {
  const id = useId();

  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button
          variant='ghost'
          size='icon'
          title='Settings'
          className='h-7 w-7 cursor-pointer'
        >
          <Settings className='h-4 w-4' />
        </Button>
      </DialogTrigger>
      <DialogContent className='sm:max-w-[425px]'>
        <DialogHeader>
          <DialogTitle>Settings</DialogTitle>
          <DialogDescription>
            Customize your editor experience with these settings.
          </DialogDescription>
        </DialogHeader>
        <div className='grid gap-4 py-4'>
          <div className='flex items-center justify-between'>
            <Label
              htmlFor={`${id}-line-numbers`}
              className='text-sm font-medium'
            >
              Line numbers
            </Label>
            <Switch
              id={`${id}-line-numbers`}
              checked={settings.lineNumbers}
              onCheckedChange={(checked) =>
                updateSettings({ lineNumbers: checked })
              }
            />
          </div>

          <div className='flex items-center justify-between'>
            <Label htmlFor={`${id}-word-wrap`} className='text-sm font-medium'>
              Word wrap
            </Label>
            <Switch
              id={`${id}-word-wrap`}
              checked={settings.lineWrapping}
              onCheckedChange={(checked) =>
                updateSettings({ lineWrapping: checked })
              }
            />
          </div>

          <div className='flex items-center justify-between'>
            <Label htmlFor={`${id}-font-size`} className='text-sm font-medium'>
              Font size
            </Label>
            <Select
              value={settings.fontSize.toString()}
              onValueChange={(value) =>
                updateSettings({ fontSize: parseInt(value) })
              }
            >
              <SelectTrigger id={`${id}-font-size`} className='w-28'>
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
          </div>

          <div className='flex items-center justify-between'>
            <Label
              htmlFor={`${id}-keybindings`}
              className='text-sm font-medium'
            >
              Keybindings
            </Label>
            <Select
              value={settings.keybindings.toString()}
              onValueChange={(value) =>
                updateSettings({ keybindings: value as 'default' | 'vim' })
              }
            >
              <SelectTrigger id={`${id}-keybindings`} className='w-28'>
                <SelectValue placeholder='Default' />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value='default'>Default</SelectItem>
                <SelectItem value='vim'>Vim</SelectItem>
              </SelectContent>
            </Select>
          </div>

          <div className='flex items-center justify-between'>
            <Label htmlFor={`${id}-tab-size`} className='text-sm font-medium'>
              Tab size
            </Label>
            <Select
              value={settings.tabSize.toString()}
              onValueChange={(value) =>
                updateSettings({ tabSize: parseInt(value) })
              }
            >
              <SelectTrigger id={`${id}-tab-size`} className='w-28'>
                <SelectValue placeholder='Tab Size' />
              </SelectTrigger>
              <SelectContent>
                {TAB_SIZES.map((size) => (
                  <SelectItem key={size} value={size.toString()}>
                    {size} spaces
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
