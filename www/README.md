The documentation site and playground use Bun. From `www`:

```sh
bun install
bun run dev
```

Use `bun run build`, `bun run test`, and `bun run lint` to check changes, and
`bun run format` to format them.

Documentation lives in `docs/**/*.md`. Directories define the hierarchy; each
needs a corresponding `.md` document. Filenames become page anchors and must be
unique, using lowercase letters, digits, and hyphens.

Frontmatter requires a positive `order`. An optional `title` defaults to the
filename without `.md`; `severity` (`error` or `warning`) adds a badge. Analysis
categories have one-word titles. Categories and rules use `order: 10` for
alphabetical sorting, and rules omit `title` to display their IDs.

Label code fences `just reported` or `just corrected` for paired examples.
