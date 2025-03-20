au VimEnter,BufWinEnter,BufRead,BufNewFile {.,}justfile\c,*.just setlocal filetype=just | setlocal commentstring=#\ %s
