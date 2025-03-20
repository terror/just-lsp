#!/usr/bin/env python3
"""Generate NeoVim queries.

Everything in `queries/` uses tree-sitter syntax, as defined at
<https://tree-sitter.github.io/tree-sitter/syntax-highlighting#theme>. However, each
editor has a slightly different syntax.

This file performs conversions so two sets of files don't need to be maintained.

<https://docs.helix-editor.com/master/themes.html#syntax-highlighting> has a bit better
guide for the standard parameters than tree-sitter does.
"""

import re
from glob import glob
from pathlib import Path

REPLACEMENTS_HELIX = []
REPLACEMENTS_ZED = []
REPLACEMENTS_LAPCE = []

ALLOWED_CAPS_ALL = ["@error"]
ALLOWED_PREDICATES_ALL = [
    "#eq?",
    "#match?",
]

ALLOWED_CAPS_NVIM = {
    "highlights.scm": [
        "@variable",
        "@variable.builtin",
        "@variable.parameter",
        "@variable.parameter.builtin",
        "@variable.member",
        "@constant",
        "@constant.builtin",
        "@constant.macro",
        "@module",
        "@module.builtin",
        "@label",
        "@string",
        "@string.documentation",
        "@string.regexp",
        "@string.escape",
        "@string.special",
        "@string.special.symbol",
        "@string.special.url",
        "@string.special.path",
        "@character",
        "@character.special",
        "@boolean",
        "@number",
        "@number.float",
        "@type",
        "@type.builtin",
        "@type.definition",
        "@type.qualifier",
        "@attribute",
        "@attribute.builtin",
        "@property",
        "@function",
        "@function.builtin",
        "@function.call",
        "@function.macro",
        "@function.method",
        "@function.method.call",
        "@constructor",
        "@operator",
        "@keyword",
        "@keyword.coroutine",
        "@keyword.function",
        "@keyword.operator",
        "@keyword.import",
        "@keyword.storage",
        "@keyword.type",
        "@keyword.repeat",
        "@keyword.return",
        "@keyword.debug",
        "@keyword.exception",
        "@keyword.conditional",
        "@keyword.conditional.ternary",
        "@keyword.directive",
        "@keyword.directive.define",
        "@punctuation.delimiter",
        "@punctuation.bracket",
        "@punctuation.special",
        "@comment",
        "@comment.documentation",
        "@comment.error",
        "@comment.warning",
        "@comment.todo",
        "@comment.note",
        "@none",
        "@conceal",
        "@spell",
        "@nospell",
    ],
    "injections.scm": [
        "@injection.language",
        "@injection.content",
        "@comment",
    ],
    "locals.scm": [
        "@local.definition",
        "@local.definition.constant",
        "@local.definition.function",
        "@local.definition.method",
        "@local.definition.var",
        "@local.definition.parameter",
        "@local.definition.macro",
        "@local.definition.type",
        "@local.definition.field",
        "@local.definition.enum",
        "@local.definition.namespace",
        "@local.definition.import",
        "@local.definition.associated",
        "@local.scope",
        "@local.reference",
    ],
    "folds.scm": [
        "@fold",
    ],
    "indents.scm": [
        "@indent.begin",
        "@indent.end",
        "@indent.align",
        "@indent.dedent",
        "@indent.branch",
        "@indent.ignore",
        "@indent.auto",
        "@indent.zero",
    ],
    "textobjects.scm": [
        "@attribute.inner",
        "@attribute.outer",
        "@function.inner",
        "@function.outer",
        "@class.inner",
        "@class.outer",
        "@conditional.inner",
        "@conditional.outer",
        "@loop.inner",
        "@loop.outer",
        "@call.inner",
        "@call.outer",
        "@block.inner",
        "@block.outer",
        "@parameter.inner",
        "@parameter.outer",
        "@regex.inner",
        "@regex.outer",
        "@comment.inner",
        "@comment.outer",
        "@assignment.inner",
        "@assignment.outer",
        "@return.inner",
        "@return.outer",
        "@frame.inner",
        "@frame.outer",
    ],
}

ALLOWED_SETTINGS_NVIM = {
    "injections.scm": [
        "injection.combined",
        "injection.language",
        "injection.include-children",
    ]
}

# Old nvim-treesitter before updates
REPLACEMENTS_NVIM = [
    # Changes to highlight queries
    (r"@comment.line", r"@comment"),
    (r"@constant.builtin.boolean", r"@boolean"),
    (r"@constant.character.escape", r"@string.escape"),
    (r"@keyword.module", r"@module"),
    (r"@keyword.control.", r"@keyword."),
    (r"@namespace", r"@module"),
    # Changes to indent queries
    (r"@indent\s+@extend", r"@indent.begin"),
    # Changes to textobject queries
    (r"(@[\w.]+\.)inside", r"\1inner"),
    (r"(@[\w.]+\.)around", r"\1outer"),
    # nvim uses `var` rather than `variable`
    (r"(@[\w.]+)\.variable", r"\1.var"),
    # nothing more specific than reference
    (r"(@local.reference)[\w.]+", r"\1"),
]

ALLOWED_CAPS_HELIX = {
    "highlights.scm": [
        "@attribute",
        "@type.builtin",
        "@type.parameter",
        "@type.enum.variant",
        "@constructor",
        "@constant.builtin",
        "@constant.builtin.boolean",
        "@constant.character",
        "@constant.character.escape",
        "@constant.numeric",
        "@constant.numeric.integer",
        "@constant.numeric.float",
        "@string",
        "@string.regexp",
        "@string.special.path",
        "@string.special.url",
        "@string.special.symbol",
        "@comment",
        "@comment.line",
        "@comment.block",
        "@comment.block.documentation",
        "@variable",
        "@variable.builtin",
        "@variable.parameter",
        "@variable.other",
        "@variable.other.member",
        "@label",
        "@punctuation",
        "@punctuation.delimiter",
        "@punctuation.bracket",
        "@punctuation.special",
        "@keyword",
        "@keyword.control",
        "@keyword.control.conditional",
        "@keyword.control.repeat",
        "@keyword.control.import",
        "@keyword.control.return",
        "@keyword.control.exception",
        "@keyword.operator",
        "@keyword.directive",
        "@keyword.function",
        "@keyword.storage",
        "@keyword.storage.type",
        "@keyword.storage.modifier",
        "@operator",
        "@function",
        "@function.builtin",
        "@function.method",
        "@function.macro",
        "@function.special",
        "@namespace",
        "@special",
    ],
    "injections.scm": [
        "@injection.language",
        "@injection.content",
        "@injection.filename",
        "@injection.shebang",
        "@comment",
    ],
    # unspecified in helix
    "locals.scm": [
        c.replace("var", "variable") for c in ALLOWED_CAPS_NVIM["locals.scm"]
    ],
    "folds.scm": ALLOWED_CAPS_NVIM["folds.scm"],
    "indents.scm": [
        "@indent",
        "@outdent",
        "@indent.always",
        "@outdent.always",
        "@align",
        "@extend",
        "@extend.prevent-once",
    ],
    "textobjects.scm": [
        "@function.inside",
        "@function.around",
        "@class.inside",
        "@class.around",
        "@test.inside",
        "@test.around",
        "@parameter.inside",
        "@parameter.around",
        "@comment.inside",
        "@comment.around",
    ],
}

ALLOWED_SETTINGS_HELIX = {
    "injections.scm": [
        "injection.combined",
        "injection.language",
        "injection.include-children",
        "injection.include-unnamed-children",
    ]
}

REPLACEMENTS_HELIX = [
    (r"@keyword.module", r"@keyword.directive"),
    (r"@function.call", r"@function"),
    (r"@spell ?", r""),
    # nothing more specific than `@local.(reference,definition)`
    (r"(@local\.\w+)\.[.\w]+", r"\1"),
]

# Documentation not yet complete
ALLOWED_CAPS_ZED = ALLOWED_CAPS_HELIX
ALLOWED_SETTINGS_ZED = ALLOWED_SETTINGS_HELIX
REPLACEMENTS_ZED = REPLACEMENTS_HELIX
ALLOWED_CAPS_LAPCE = ALLOWED_CAPS_HELIX
ALLOWED_SETTINGS_LAPCE = ALLOWED_SETTINGS_HELIX
REPLACEMENTS_LAPCE = REPLACEMENTS_HELIX

# (rname, eplacements, allowed caps, base path) mappings to create a flavor
FLAVOR_MAPPINGS = [
    (
        "new NeoVim",
        "NVIM",
        REPLACEMENTS_NVIM,
        ALLOWED_CAPS_NVIM,
        ALLOWED_SETTINGS_NVIM,
        Path("queries") / "just",
    ),
    (
        "Helix",
        "HELIX",
        REPLACEMENTS_HELIX,
        ALLOWED_CAPS_HELIX,
        ALLOWED_SETTINGS_HELIX,
        Path("queries-flavored") / "helix",
    ),
    (
        "Zed",
        "ZED",
        REPLACEMENTS_ZED,
        ALLOWED_CAPS_ZED,
        ALLOWED_SETTINGS_ZED,
        Path("queries-flavored") / "zed",
    ),
    (
        "Lapce",
        "LAPCE",
        REPLACEMENTS_LAPCE,
        ALLOWED_CAPS_LAPCE,
        ALLOWED_SETTINGS_LAPCE,
        Path("queries-flavored") / "lapce",
    ),
]


def main():
    sources = glob("queries-src/*.scm")
    for fname in sources:
        qname = Path(fname).name
        print(f"Generating flavored queries from {fname}")
        base_contents = "; File autogenerated by build-queries-nvim.py; do not edit\n\n"

        with open(fname) as f:
            base_contents += f.read()

        for (
            name,
            tag,
            replacements,
            allowed_caps,
            allowed_settings,
            basepath,
        ) in FLAVOR_MAPPINGS:
            # Remove lines as indicated by directives
            contents = "\n".join(
                (
                    line
                    for line in base_contents.splitlines()
                    if f"SKIP-{tag}" not in line
                )
            )

            # Delete other directives
            contents = re.sub(r";?\s*SKIP-[\w-]+", "", contents)

            for rep in replacements:
                pat = rep[0]
                sub = rep[1]
                flags = 0
                if len(rep) > 2:
                    flags = rep[2]
                contents = re.sub(pat, sub, contents, flags=flags)

            # Remove trailing whitespace and duplicate newlines
            contents = re.sub(r"[\s;]+$", "", contents)
            contents = "\n".join((line.rstrip() for line in contents.splitlines()))
            contents = re.sub(r"((?:\r?\n){2,})(?:\r?\n)+", r"\1", contents)

            if not contents.endswith("\n"):
                contents += "\n"

            basepath.mkdir(parents=True, exist_ok=True)
            dest = basepath / Path(fname).name

            with open(dest, "w") as f:
                f.write(contents)

            # Validate all captures are valid
            for qcap in re.finditer(r"@[\w.]+", contents):
                matched = qcap[0]

                # Internal captures
                if matched.startswith("@_"):
                    continue

                allowed = allowed_caps[qname]
                assert (
                    matched in allowed or matched in ALLOWED_CAPS_ALL
                ), f"found disallowed query '{matched}' in '{dest}' ({name}). Allowed: {allowed}"

            # Validate all settings are valid
            for qsetting in re.finditer(r"#set!\s+([\w.-]+)", contents):
                matched = qsetting[1]
                allowed = allowed_settings[qname]
                assert (
                    matched in allowed
                ), f"found disallowed setting '{qsetting[0]}' in '{dest}' ({name}). Allowed: {allowed}"


if __name__ == "__main__":
    main()
