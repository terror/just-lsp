#!/usr/bin/env python3
"""Turn our tests into a corpus for the fuzzer (one test per file)"""

from pathlib import Path
import re

RE = r"===+\n(?P<name>.*?)\n===+\n(?P<source>.*?)\n---+"
REPO = Path(__file__).parent.parent
OUT_DIR = REPO / "fuzzer" / "corpus"


def main():
    ts_corpus_files = (REPO / "test" / "corpus").glob("*.txt")

    corpus = {}

    for fname in ts_corpus_files:
        text = fname.read_text()
        prefix = fname.name.rstrip(".txt")

        for match in re.finditer(RE, text, re.MULTILINE | re.DOTALL):
            name = match.group("name").replace(" ", "_")
            name = f"{prefix}_{name}.just"
            source = match.group("source")
            corpus[name] = source

    OUT_DIR.mkdir(exist_ok=True)

    # Clear the corpus of all files we created
    for existing in OUT_DIR.iterdir():
        if existing.name.endswith(".just"):
            existing.unlink()

    for name, source in corpus.items():
        out_file: Path = OUT_DIR / name
        out_file.write_text(source)


if __name__ == "__main__":
    main()
