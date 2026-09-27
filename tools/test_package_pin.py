"""Unit tests for tools/package-pin: `python3 -m unittest discover -s tools`."""
import importlib.machinery
import importlib.util
import os
import pathlib
import subprocess
import tempfile
import textwrap
import unittest

# Git exports GIT_DIR, GIT_WORK_TREE and GIT_INDEX_FILE into hook processes and they
# OVERRIDE `git -C <dir>`; check_cmd runs this suite from the pre-commit hook, so a
# fixture's commits would otherwise land in THIS repository. Same scrub as
# test_upstream_report.py.
for _name in [_key for _key in os.environ if _key.startswith("GIT_")]:
    del os.environ[_name]

spec = importlib.util.spec_from_loader(
    "package_pin",
    importlib.machinery.SourceFileLoader(
        "package_pin", str(pathlib.Path(__file__).with_name("package-pin"))
    ),
)
package_pin = importlib.util.module_from_spec(spec)
spec.loader.exec_module(package_pin)

PKGBUILD = textwrap.dedent("""\
    pkgname=niri-material
    pkgver=26.04.r7.gdeadbeef
    pkgrel=1
    source=("niri::git+https://github.com/khughitt/niri-material.git#commit=%s")

    build() {
      cd "$srcdir/niri"
      export NIRI_BUILD_COMMIT=deadbeef
    }
    """)


def run(cwd, *args):
    subprocess.run(args, cwd=cwd, check=True, capture_output=True, text=True)


class PinTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        self.addCleanup(self.tmp.cleanup)
        run(self.root, "git", "init", "-q", "-b", "main")
        run(self.root, "git", "config", "user.email", "test@example.com")
        run(self.root, "git", "config", "user.name", "Test")
        (self.root / "docs/materials").mkdir(parents=True)
        (self.root / "packaging/arch").mkdir(parents=True)
        (self.root / "seed").write_text("seed\n")
        run(self.root, "git", "add", ".")
        run(self.root, "git", "commit", "-qm", "baseline")
        self.baseline = subprocess.run(
            ("git", "rev-parse", "HEAD"), cwd=self.root, capture_output=True, text=True,
        ).stdout.strip()
        (self.root / "docs/materials/upstream-baseline.toml").write_text(
            f'tag = "v26.04"\npatched_commit = "{self.baseline}"\n')
        self.commits = []
        for index in range(3):
            (self.root / "seed").write_text(f"seed {index}\n")
            run(self.root, "git", "add", ".")
            run(self.root, "git", "commit", "-qm", f"work {index}")
            self.commits.append(subprocess.run(
                ("git", "rev-parse", "HEAD"), cwd=self.root, capture_output=True, text=True,
            ).stdout.strip())

    def write_pkgbuild(self, commit):
        (self.root / package_pin.PKGBUILD).write_text(PKGBUILD % commit)

    def test_the_count_is_the_commits_this_fork_carries_over_the_baseline(self):
        pin = package_pin.pin_for(self.root, self.commits[2])
        self.assertEqual(pin["pkgver"], f"26.04.r3.g{self.commits[2][:8]}")
        self.assertEqual(pin["source_commit"], self.commits[2])
        self.assertEqual(pin["build_commit"], self.commits[2][:8])

        earlier = package_pin.pin_for(self.root, self.commits[0])
        self.assertEqual(earlier["pkgver"], f"26.04.r1.g{self.commits[0][:8]}")

    def test_writing_a_pin_replaces_all_three_values(self):
        self.write_pkgbuild(self.commits[0])
        pin = package_pin.pin_for(self.root, self.commits[2])

        written = package_pin.write_pin(
            (self.root / package_pin.PKGBUILD).read_text(), pin)

        self.assertEqual(package_pin.read_pin(written), pin)

    def test_check_accepts_a_pkgbuild_that_describes_its_own_commit(self):
        self.write_pkgbuild(self.commits[1])
        path = self.root / package_pin.PKGBUILD
        path.write_text(package_pin.write_pin(
            path.read_text(), package_pin.pin_for(self.root, self.commits[1])))

        self.assertEqual(self.check(), 0)

    def test_check_rejects_a_hand_edited_count(self):
        self.write_pkgbuild(self.commits[1])
        path = self.root / package_pin.PKGBUILD
        text = package_pin.write_pin(
            path.read_text(), package_pin.pin_for(self.root, self.commits[1]))
        path.write_text(text.replace(".r2.", ".r9."))

        self.assertEqual(self.check(), 1)

    def check(self):
        result = subprocess.run(
            (str(pathlib.Path(package_pin.__file__)), "--check"),
            cwd=self.root, capture_output=True, text=True)
        return result.returncode


if __name__ == "__main__":
    unittest.main()
