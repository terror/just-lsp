import importlib.metadata
import os
import subprocess
import sysconfig
import unittest
from pathlib import Path


class Package(unittest.TestCase):
  def test_executable(self):
    executable = "just-lsp.exe" if os.name == "nt" else "just-lsp"

    output = subprocess.run(
      [str(Path(sysconfig.get_path("scripts")) / executable), "--version"],
      capture_output=True,
      check=True,
      encoding="utf-8",
    )

    self.assertEqual(
      output.stdout, f"just-lsp {importlib.metadata.version('just-lsp')}\n"
    )

    self.assertEqual(output.stderr, "")


if __name__ == "__main__":
  unittest.main()
