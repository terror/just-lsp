The documentation site and playground use Bun. From `www`, install dependencies
with `bun install`, start the development server with `bun run dev`, and build
with `bun run build`.

React Router serves the home page at `/`, the documentation at `/documentation`,
and the playground at `/playground`. The configuration overview lives in
`docs/configuration.md`, with subsections in `docs/configuration/*.md` at the
repository root. Configuration subsections are listed in
`src/pages/documentation.tsx`, which uses the same entries for navigation and
headings.

The rule overview lives in `docs/rules.md`, and individual rules are discovered
from `docs/rules/*.md`. Rule files use YAML frontmatter with `title`,
`category`, `severity`, and `order` fields, followed by Markdown content. Label
code fences with `reported` or `corrected` after the language to display paired
examples.

Run unit tests with `bun run test`.

TypeScript 7 is installed as `@typescript/native` and provides the `tsc` used by
the build. The `typescript` alias points to `@typescript/typescript6`, which
provides the compiler API required by ESLint. This follows Microsoft's
[compatibility setup](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/#running-side-by-side-with-typescript-6.0).
