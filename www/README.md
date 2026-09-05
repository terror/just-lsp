The web playground uses Bun. Install dependencies with `bun install`, start the
development server with `bun run dev`, and build with `bun run build`.

Run unit tests with `bun run test`. Browser integration tests build and serve
the production app with `bun run test:integration`; install Chromium first with
`bunx playwright install chromium`.

TypeScript 7 is installed as `@typescript/native` and provides the `tsc` used by
the build. The `typescript` alias points to `@typescript/typescript6`, which
provides the compiler API required by ESLint. This follows Microsoft's
[compatibility setup](https://devblogs.microsoft.com/typescript/announcing-typescript-7-0/#running-side-by-side-with-typescript-6.0).
