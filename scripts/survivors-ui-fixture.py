#!/usr/bin/env python3
"""Copy a deterministic preview account into a new, isolated BENILLA_HOME."""

import argparse
import os
from pathlib import Path
import shlex
import shutil
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "crates/benilla-app/testdata/survivors-ui"
PROFILES = ("fresh", "partial", "collected")
PAGES = ("starter", "heroes", "talents", "rewards", "collection", "codex",
         "achievements", "notes", "drops", "look")


def prepare(profile, output=None):
    """Never replace an existing directory, even if it looks like a preview."""
    if profile not in PROFILES:
        raise ValueError(f"Unknown fixture: {profile}")
    source = FIXTURES / f"{profile}.txt"
    if not source.is_file():
        raise FileNotFoundError(source)
    if output is None:
        parent = ROOT / "target/ui-fixtures"
        parent.mkdir(parents=True, exist_ok=True)
        home = Path(tempfile.mkdtemp(prefix=f"{profile}-", dir=parent))
    else:
        home = Path(os.path.abspath(output))
        home.mkdir(parents=True, exist_ok=False)
    config = home / "benilla-config"
    config.mkdir()
    shutil.copyfile(source, config / "survivors-account.txt")
    (home / "UI-FIXTURE.txt").write_text(
        f"Synthetic Survivors UI account: {profile}\n"
        "Changes here are disposable. This is not player progress or content acceptance.\n",
        encoding="utf-8",
    )
    return home


def command(home, page):
    return " ".join([
        "env", "-u", "WOW_SURVIVORS_AUTO", "-u", "WOW_SURVIVORS_UI_PREVIEW",
        "-u", "WOW_CAPTURE",
        shlex.quote(f"WOW_DATA={ROOT / 'wow-data/Data'}"),
        shlex.quote(f"BENILLA_HOME={home}"),
        shlex.quote(f"WOW_SURVIVORS_UI_PAGE={page}"),
        "cargo", "survivors",
    ])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profile", choices=PROFILES)
    parser.add_argument("--output", type=Path, help="New directory only; defaults to target/ui-fixtures")
    parser.add_argument("--page", choices=PAGES, default="collection")
    args = parser.parse_args()
    try:
        home = prepare(args.profile, args.output)
    except (OSError, ValueError) as error:
        parser.error(str(error))
    print(f"Prepared synthetic {args.profile} account: {home}")
    print("Run from the repository root:")
    print(command(home, args.page))


if __name__ == "__main__":
    main()
