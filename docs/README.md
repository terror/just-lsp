# Documentation

The website loads the rule overview from `rules.md` and discovers individual
rules in `rules/*.md`. Each rule’s filename is its diagnostic code and its
anchor on `/documentation`.

Rule files use YAML frontmatter for their title, category, default severity,
and display order. Categories appear in the order of their first rule; rules
within each category follow `order`, with filenames breaking ties. Existing
orders leave gaps for inserting new entries.

The body is ordinary Markdown. Use `##` headings for sections, inline code for
identifiers, and fenced code blocks for examples. Mark each example pair with
`just reported` and `just corrected`; these labels produce the side-by-side
examples on the website. Keep each pair in that order. Additional code blocks can use their
normal language labels, such as `json`.

For example, `rules/syntax-errors.md` has this structure:

````markdown
---
title: Syntax errors
category: Syntax and imports
severity: error
order: 10
---

Reports malformed justfile syntax.

- Recipe headers must end with a colon.

## How to fix it

Add the missing colon.

## Examples

```just reported
foo
  echo foo
```

```just corrected
foo:
  echo foo
```
````
