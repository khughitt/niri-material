"""Run git in tooling tests without the host's git environment or configuration.

Git hooks export GIT_DIR, GIT_WORK_TREE and GIT_INDEX_FILE, which override `git -C`, so
a fixture run from a hook would write to the repository being committed to. A host's
global or system config can declare hooks (config hooks, core.hooksPath) and settings
that every fixture commit would then run; CI has neither. Design:
docs/specs/2026-10-17-pre-commit-fast-hermetic-git-design.md.
"""
import os
from collections.abc import MutableMapping


def isolate(environ: MutableMapping[str, str]) -> None:
    """Drop every GIT_* variable, then read no global or system git config."""
    for name in [key for key in environ if key.startswith('GIT_')]:
        del environ[name]
    environ['GIT_CONFIG_GLOBAL'] = os.devnull
    environ['GIT_CONFIG_NOSYSTEM'] = '1'
