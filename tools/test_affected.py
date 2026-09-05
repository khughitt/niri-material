"""Unit tests for tools/test-affected: `python3 -m unittest discover -s tools`."""
import importlib.util
import pathlib
import unittest

spec = importlib.util.spec_from_loader(
    "test_affected", importlib.machinery.SourceFileLoader("test_affected", str(pathlib.Path(__file__).with_name("test-affected")))
)
affected = importlib.util.module_from_spec(spec)
spec.loader.exec_module(affected)

MEMBERS = {"niri": ".", "niri-config": "niri-config", "niri-ipc": "niri-ipc", "niri-visual-tests": "niri-visual-tests"}
DEPENDENTS = {
    "niri": {"niri-visual-tests"},
    "niri-config": {"niri", "niri-visual-tests"},
    "niri-ipc": {"niri-config", "niri"},
    "niri-visual-tests": set(),
}


class Select(unittest.TestCase):
    def select(self, *paths):
        return affected.select(list(paths), MEMBERS, DEPENDENTS)

    def test_nothing_changed_selects_nothing(self):
        self.assertEqual(self.select(), [])

    def test_docs_and_tasks_select_nothing(self):
        self.assertEqual(self.select("docs/plans/x.md", "tasks/material-1.md", "README.md", ".github/workflows/ci.yml"), [])

    def test_root_source_selects_the_root_package(self):
        self.assertEqual(self.select("src/layout/mod.rs"), ["niri"])
        self.assertEqual(self.select("resources/default-config.kdl"), ["niri"])
        self.assertEqual(self.select("build.rs"), ["niri"])

    def test_member_change_selects_its_dependents(self):
        self.assertEqual(self.select("niri-config/src/lib.rs"), ["niri", "niri-config"])
        self.assertEqual(self.select("niri-ipc/Cargo.toml"), ["niri", "niri-config", "niri-ipc"])

    def test_excluded_member_is_never_selected(self):
        self.assertEqual(self.select("niri-visual-tests/src/main.rs"), [])

    def test_workspace_files_select_everything(self):
        for path in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".cargo/config.toml"):
            self.assertEqual(self.select(path), ["niri", "niri-config", "niri-ipc"], path)


if __name__ == "__main__":
    unittest.main()
