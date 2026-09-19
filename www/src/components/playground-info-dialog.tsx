import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogTitle,
  DialogTrigger,
} from '@/components/ui/dialog';
import { ExternalLink, Info } from 'lucide-react';

const repository = 'https://github.com/terror/just-lsp';

export const PlaygroundInfoDialog = () => (
  <Dialog>
    <DialogTrigger asChild>
      <Button
        variant='ghost'
        size='icon'
        className='h-7 w-7 cursor-pointer'
        aria-label='Information'
        title='Information'
      >
        <Info className='h-4 w-4' aria-hidden='true' />
      </Button>
    </DialogTrigger>
    <DialogContent
      aria-describedby={undefined}
      className='max-h-[calc(100dvh-2rem)] overflow-y-auto sm:max-w-[520px]'
    >
      <DialogTitle className='sr-only'>Parser information</DialogTitle>
      <dl className='grid gap-4 text-sm'>
        <div className='grid gap-1'>
          <dt className='text-muted-foreground text-xs font-medium'>Asset</dt>
          <dd className='font-mono text-xs break-all'>
            {import.meta.env.BASE_URL}tree-sitter-just.wasm
          </dd>
        </div>
        <div className='grid gap-1'>
          <dt className='text-muted-foreground text-xs font-medium'>Path</dt>
          <dd className='font-mono text-xs break-all'>
            vendor/tree-sitter-just
          </dd>
        </div>
        <div className='grid gap-1'>
          <dt className='text-muted-foreground text-xs font-medium'>
            Repository
          </dt>
          <dd className='min-w-0'>
            <a
              href={repository}
              target='_blank'
              rel='noreferrer'
              className='inline-flex max-w-full cursor-pointer items-center gap-1 underline-offset-4 hover:underline'
            >
              <span className='truncate'>terror/just-lsp</span>
              <ExternalLink className='h-3 w-3 shrink-0' aria-hidden='true' />
            </a>
          </dd>
        </div>
        <div className='grid gap-1'>
          <dt className='text-muted-foreground text-xs font-medium'>
            Revision
          </dt>
          <dd>
            <a
              href={`${repository}/commit/${import.meta.env.VITE_GIT_REVISION}`}
              target='_blank'
              rel='noreferrer'
              className='cursor-pointer font-mono text-xs break-all underline-offset-4 hover:underline'
            >
              {import.meta.env.VITE_GIT_REVISION}
            </a>
          </dd>
        </div>
      </dl>
    </DialogContent>
  </Dialog>
);
