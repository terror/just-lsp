import { Button } from '@/components/ui/button';
import { Dialog, DialogContent, DialogTrigger } from '@/components/ui/dialog';

export const AboutDialog = () => {
  return (
    <Dialog>
      <DialogTrigger asChild>
        <Button
          variant='ghost'
          size='sm'
          title='About'
          className='h-8 cursor-pointer px-2 font-bold hover:bg-transparent hover:text-inherit'
        >
          about
        </Button>
      </DialogTrigger>

      <DialogContent className='sm:max-w-[520px]'>
        <div className='text-muted-foreground space-y-4 text-sm leading-6'>
          <p>
            <span className='font-semibold'>just-lsp</span> implements the
            Language Server Protocol for{' '}
            <a href='https://github.com/casey/just'>just</a>, the command
            runner. The server provides editor features for justfiles, including
            completions, hover information, definitions, diagnostics,
            references, rename support, code actions, semantic highlighting,
            folding, and formatting.
          </p>

          <p>
            This playground runs the just grammar in the browser with
            tree-sitter. The editor pane contains an editable justfile, and the
            syntax tree pane shows the parsed structure for the current
            document.
          </p>

          <div className='grid gap-2 pt-2'>
            <a
              href='https://github.com/terror/just-lsp'
              className='text-foreground font-medium underline-offset-4 hover:underline'
              target='_blank'
              rel='noreferrer'
            >
              View the project on GitHub
            </a>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
};
