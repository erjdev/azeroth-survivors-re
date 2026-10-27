#!/usr/bin/env python3
"""Copy a deterministic preview account into a new, isolated BENILLA_HOME."""

import argparse
import os
from pathlib import Path
import shlex
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "crates/benilla-app/testdata/survivors-ui"
PREVIEWS = ROOT / "docs/ui-previews"
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
        "-u", "WOW_CAPTURE", "-u", "WOW_SURVIVORS_UI_PAGE",
        shlex.quote(f"WOW_DATA={ROOT / 'wow-data/Data'}"),
        shlex.quote(f"BENILLA_HOME={home}"),
        shlex.quote(f"WOW_SURVIVORS_UI_PAGE={page}"),
        "cargo", "survivors",
    ])


def preview(home, name):
    """Copy a repository preset; later edits affect only the isolated request."""
    if Path(name).name != name or name not in {p.name for p in PREVIEWS.glob("*.txt")}:
        raise ValueError(f"Unknown preview preset: {name}")
    destination = home / "view.txt"
    with destination.open("xb") as output:
        output.write((PREVIEWS / name).read_bytes())
    return destination


def launch_environment(home, page, request=None):
    env = os.environ.copy()
    for key in ("WOW_SURVIVORS_AUTO", "WOW_CAPTURE", "WOW_SURVIVORS_UI_PREVIEW",
                "WOW_SURVIVORS_UI_PAGE"):
        env.pop(key, None)
    env["WOW_DATA"] = str(ROOT / "wow-data/Data")
    env["BENILLA_HOME"] = str(home)
    if request is None:
        env["WOW_SURVIVORS_UI_PAGE"] = page
    else:
        env["WOW_SURVIVORS_UI_PREVIEW"] = str(request)
    return env


def launch(home, page, request=None):
    # No shell: paths with spaces and Windows drive letters remain single arguments.
    return subprocess.run(
        ["cargo", "survivors"], cwd=ROOT,
        env=launch_environment(home, page, request), check=False,
    ).returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("profile", choices=PROFILES)
    parser.add_argument("--output", type=Path, help="New directory only; defaults to target/ui-fixtures")
    parser.add_argument("--page", choices=PAGES, default="collection")
    parser.add_argument("--preset", choices=sorted(p.name for p in PREVIEWS.glob("*.txt")),
                        help="Copy a watched UI request; takes precedence over --page")
    parser.add_argument("--run", action="store_true",
                        help="Build incrementally and launch the isolated preview (all platforms)")
    args = parser.parse_args()
    try:
        home = prepare(args.profile, args.output)
        request = preview(home, args.preset) if args.preset else None
    except (OSError, ValueError) as error:
        parser.error(str(error))
    print(f"Prepared synthetic {args.profile} account: {home}")
    if request:
        print(f"Live preview request: {request} (edit this file while the app is open)")
    if args.run:
        try:
            return launch(home, args.page, request)
        except OSError as error:
            parser.error(f"Could not launch cargo: {error}. Fixture retained at {home}")
    print("Add --run to launch directly on Windows, macOS or Linux.")
    if os.name != "nt":
        launch_command = command(home, args.page)
        if request:
            launch_command = launch_command.replace(
                shlex.quote(f"WOW_SURVIVORS_UI_PAGE={args.page}"),
                shlex.quote(f"WOW_SURVIVORS_UI_PREVIEW={request}"),
            )
        print("Or run this prepared fixture from the repository root:")
        print(launch_command)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
