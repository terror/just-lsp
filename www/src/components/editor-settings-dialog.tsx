import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
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
import { useEditorSettings } from '@/providers/editor-settings-provider';
import { Settings } from 'lucide-react';
import { useState } from 'react';

export const EditorSettingsDialog = () => {
  const { settings, updateSettings } = useEditorSettings();

  const [settingsOpen, setSettingsOpen] = useState<boolean>(false);

  return (
    <>
      <Button
        variant='ghost'
        size='icon'
        onClick={() => setSettingsOpen(true)}
        title='Settings'
        className='h-7 w-7 cursor-pointer'
      >
        <Settings className='h-4 w-4' />
      </Button>

      <Dialog open={settingsOpen} onOpenChange={setSettingsOpen}>
        <DialogContent className='sm:max-w-[425px]'>
          <DialogHeader>
            <DialogTitle>Settings</DialogTitle>
            <DialogDescription>
              Customize your editor experience with these settings.
            </DialogDescription>
          </DialogHeader>
          <div className='grid gap-4 py-4'>
            <div className='flex items-center justify-between'>
              <Label className='text-sm font-medium'>Line numbers</Label>
              <Switch
                checked={settings.lineNumbers}
                onCheckedChange={(checked) =>
                  updateSettings({ lineNumbers: checked })
                }
              />
            </div>

            <div className='flex items-center justify-between'>
              <Label className='text-sm font-medium'>Word wrap</Label>
              <Switch
                checked={settings.lineWrapping}
                onCheckedChange={(checked) =>
                  updateSettings({ lineWrapping: checked })
                }
              />
            </div>

            <div className='flex items-center justify-between'>
              <Label className='text-sm font-medium'>Font size</Label>
              <Select
                value={settings.fontSize.toString()}
                onValueChange={(value) =>
                  updateSettings({ fontSize: parseInt(value) })
                }
              >
                <SelectTrigger className='w-28'>
                  <SelectValue placeholder='Font size' />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='12'>12px</SelectItem>
                  <SelectItem value='14'>14px</SelectItem>
                  <SelectItem value='16'>16px</SelectItem>
                  <SelectItem value='18'>18px</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className='flex items-center justify-between'>
              <Label className='text-sm font-medium'>Keybindings</Label>
              <Select
                value={settings.keybindings.toString()}
                onValueChange={(value) =>
                  updateSettings({ keybindings: value as 'default' | 'vim' })
                }
              >
                <SelectTrigger className='w-28'>
                  <SelectValue placeholder='Default' />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='default'>Default</SelectItem>
                  <SelectItem value='vim'>Vim</SelectItem>
                </SelectContent>
              </Select>
            </div>

            <div className='flex items-center justify-between'>
              <Label className='text-sm font-medium'>Tab size</Label>
              <Select
                value={settings.tabSize.toString()}
                onValueChange={(value) =>
                  updateSettings({ tabSize: parseInt(value) })
                }
              >
                <SelectTrigger className='w-28'>
                  <SelectValue placeholder='Tab Size' />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value='2'>2 spaces</SelectItem>
                  <SelectItem value='4'>4 spaces</SelectItem>
                  <SelectItem value='8'>8 spaces</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </>
  );
};
