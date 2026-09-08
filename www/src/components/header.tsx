import { Bot, Moon, Sun } from 'lucide-react';
import { NavLink } from 'react-router';

import { type useTheme } from '../hooks/use-theme';
import { Button } from './ui/button';

export const Header = ({ theme }: { theme: ReturnType<typeof useTheme> }) => (
  <header className='flex flex-wrap items-center gap-x-6 gap-y-2 px-4 py-3 sm:px-6'>
    <NavLink to='/' end className='flex items-center gap-2 font-semibold'>
      <Bot className='h-4 w-4' aria-hidden='true' />
      just-lsp
    </NavLink>
    <nav
      aria-label='Main navigation'
      className='ml-auto flex flex-wrap items-center gap-4 text-sm'
    >
      <NavLink
        to='/playground'
        className='text-muted-foreground hover:text-foreground aria-[current=page]:text-foreground aria-[current=page]:font-medium'
      >
        Playground
      </NavLink>
      <a
        href='https://github.com/terror/just-lsp'
        className='text-muted-foreground hover:text-foreground'
      >
        GitHub
      </a>
      <Button
        variant='ghost'
        size='icon'
        className='h-8 w-8 cursor-pointer'
        onClick={theme.toggleTheme}
        aria-label={
          theme.darkMode ? 'Switch to light mode' : 'Switch to dark mode'
        }
        aria-pressed={theme.darkMode}
        title={theme.darkMode ? 'Switch to light mode' : 'Switch to dark mode'}
      >
        {theme.darkMode ? <Sun /> : <Moon />}
      </Button>
    </nav>
  </header>
);
