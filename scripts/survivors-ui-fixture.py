#!/usr/bin/env python3
"""Copy a deterministic preview account into a new, isolated BENILLA_HOME."""

import argparse
import os
from pathlib import Path
import shlex
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
FIXTURES = ROOT / "crates/benilla-app/testdata/survivors-ui"
PREVIEWS = ROOT / "docs/ui-previews"
PROFILES = ("fresh", "partial", "collected", "trained")
PAGES = ("starter", "heroes", "talents", "rewards", "collection", "codex",
         "achievements", "notes", "drops", "look")


def prepare(profile, output=None, hero=None, selected_spec=None):
    """Never replace an existing directory, even if it looks like a preview."""
    if profile not in PROFILES:
        raise ValueError(f"Unknown fixture: {profile}")
    source = FIXTURES / f"{profile}.txt"
    if not source.is_file():
        raise FileNotFoundError(source)
    contents = source.read_bytes().decode("utf-8")
    if (hero is None) != (selected_spec is None):
        raise ValueError("Hero and spec must be selected together")
    if hero is not None:
        if hero <= 0 or selected_spec not in (1, 2, 3):
            raise ValueError("Hero must be positive; spec must be 1, 2 or 3")
        lines = contents.splitlines(keepends=True)
        matches = [i for i, line in enumerate(lines)
                   if line.startswith(f"who {hero} ")]
        if len(matches) != 1:
            raise ValueError(f"Hero {hero} is not in the {profile} fixture")
        index = matches[0]
        fields = lines[index].split(maxsplit=6)
        _, separator, spreads = fields[6].partition("|")
        if not separator:
            raise ValueError("Invalid fixture hero record")
        lines[index] = " ".join(fields[:6]) + f" {selected_spec - 1}|" + spreads
        contents = "".join(lines)
    if output is None:
        parent = ROOT / "target/ui-fixtures"
        parent.mkdir(parents=True, exist_ok=True)
        home = Path(tempfile.mkdtemp(prefix=f"{profile}-", dir=parent))
    else:
        home = Path(os.path.abspath(output))
        home.mkdir(parents=True, exist_ok=False)
    config = home / "benilla-config"
    config.mkdir()
    (config / "survivors-account.txt").write_bytes(contents.encode("utf-8"))
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


def preview_contents(name, hero=None):
    """Validate and read a preset before creating any disposable state."""
    if Path(name).name != name or name not in {p.name for p in PREVIEWS.glob("*.txt")}:
        raise ValueError(f"Unknown preview preset: {name}")
    contents = (PREVIEWS / name).read_bytes().decode("utf-8")
    if hero is not None:
        lines = contents.splitlines()
        if not any(line.split("#", 1)[0].strip().replace(" ", "") == "page=talents" for line in lines):
            raise ValueError("Saved spec selection requires a talent preview preset")
        lines = [line for line in lines if line.split("=", 1)[0].strip() != "hero"]
        contents = "\n".join(lines) + f"\nhero={hero}\n"
    return contents.encode("utf-8")


def write_preview(home, contents):
    destination = home / "view.txt"
    with destination.open("xb") as output:
        output.write(contents)
    return destination


def preview(home, name, hero=None):
    """Copy a repository preset; later edits affect only the isolated request."""
    return write_preview(home, preview_contents(name, hero))


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


def existing_executable(path):
    """Resolve before creating fixture state; do not infer a build's freshness."""
    executable = path.expanduser().resolve(strict=True)
    if not executable.is_file():
        raise ValueError(f"Not an executable file: {executable}")
    if not os.access(executable, os.X_OK):
        raise ValueError(f"File is not executable: {executable}")
    return executable


def launch(home, page, request=None, executable=None):
    # No shell: paths with spaces and Windows drive letters remain single arguments.
    return subprocess.run(
        ([str(executable)] if executable is not None else ["cargo", "survivors"]), cwd=ROOT,
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
    parser.add_argument("--executable", type=Path,
                        help="With --run, launch this existing native development executable without Cargo; rebuild Rust changes first")
    parser.add_argument("--hero", type=int, help="Fixture hero id; requires --spec and a talent --preset")
    parser.add_argument("--spec", type=int, choices=(1, 2, 3), help="Saved spec in the disposable fixture")
    args = parser.parse_args()
    if (args.hero is None) != (args.spec is None) or (args.spec is not None and not args.preset):
        parser.error("--hero and --spec require each other and a talent --preset")
    if args.executable is not None and not args.run:
        parser.error("--executable requires --run")
    try:
        executable = existing_executable(args.executable) if args.executable is not None else None
        contents = preview_contents(args.preset, args.hero) if args.preset else None
        home = prepare(args.profile, args.output, args.hero, args.spec)
        request = write_preview(home, contents) if contents is not None else None
    except (OSError, ValueError) as error:
        parser.error(str(error))
    print(f"Prepared synthetic {args.profile} account: {home}")
    if args.spec is not None:
        print(f"Selected fixture hero {args.hero}, saved spec {args.spec}")
    if request:
        print(f"Live preview request: {request} (edit this file while the app is open)")
    if args.run:
        try:
            if executable is not None:
                print(f"Launching existing executable without rebuilding: {executable}", flush=True)
                print("Rust changes are not checked; rebuild before validating changed code.", flush=True)
            return launch(home, args.page, request, executable)
        except OSError as error:
            parser.error(f"Could not launch {executable or 'cargo'}: {error}. Fixture retained at {home}")
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
