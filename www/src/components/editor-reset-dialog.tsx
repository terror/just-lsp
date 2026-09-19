import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { RotateCcw } from 'lucide-react';

interface EditorResetDialogProps {
  onReset: () => void;
  sample: string;
}

export const EditorResetDialog = ({
  onReset,
  sample,
}: EditorResetDialogProps) => (
  <Dialog>
    <DialogTrigger asChild>
      <Button
        variant='ghost'
        size='icon'
        className='h-7 w-7 cursor-pointer'
        aria-label={`Reset to ${sample} sample`}
        title={`Reset to ${sample} sample`}
      >
        <RotateCcw className='h-4 w-4' aria-hidden='true' />
      </Button>
    </DialogTrigger>
    <DialogContent className='sm:max-w-[425px]'>
      <DialogHeader>
        <DialogTitle>Reset editor?</DialogTitle>
        <DialogDescription>
          This will replace your current editor contents with the {sample}{' '}
          sample justfile.
        </DialogDescription>
      </DialogHeader>
      <DialogFooter>
        <DialogClose asChild>
          <Button variant='outline' className='cursor-pointer'>
            Cancel
          </Button>
        </DialogClose>
        <DialogClose asChild>
          <Button
            variant='destructive'
            className='cursor-pointer'
            onClick={onReset}
          >
            Reset
          </Button>
        </DialogClose>
      </DialogFooter>
    </DialogContent>
  </Dialog>
);
