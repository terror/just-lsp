The documentation site and playground use Bun. From `www`, install dependencies
with `bun install`, start the development server with `bun run dev`, and build
with `bun run build`.

React Router serves the home page at `/`, the documentation at `/documentation`,
and the playground at `/playground`. Documentation is discovered recursively
from `docs/**/*.md` at the repository root. Every file uses YAML frontmatter
with a `title` and a positive integer `order`, followed by optional Markdown
content. Sibling entries are sorted by `order`, then filename.

The directory structure defines the hierarchy: `docs/configuration.md` is a
top-level section, and `docs/configuration/formatting.md` is one of its
subsections. Each subdirectory must have a corresponding Markdown document.
Adding a document automatically adds its content and navigation entry. A parent
document can contain only frontmatter to provide a heading without introductory
text.

Features are documented under `docs/features`. Analysis rules use the same
structure: `docs/features/analysis/aliases.md` provides the category heading,
and `docs/features/analysis/aliases/*.md` contains its rules. An optional
`severity` (`error` or `warning`) displays a default severity badge and the
entry's filename as its rule code.

Filenames without `.md` become page anchors and must be unique across the
documentation. Filenames and directories must start with a lowercase letter and
contain only lowercase letters, digits, and hyphens. Label code fences with
`reported` or `corrected` after the language to display paired examples.

Run unit tests with `bun run test`.

TypeScript 7 is installed as `@typescript/native` and provides the `tsc` used by
the build. The `typescript` alias points to `@typescript/typescript6`, which
provides the compiler API required by ESLint. This follows Microsoft's
[compatibility setup](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/#running-side-by-side-with-typescript-6.0).
