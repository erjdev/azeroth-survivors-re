#!/usr/bin/env python3
"""Read-only Cargo output budget check; run before batches and at milestones."""

import argparse
import os
from pathlib import Path
import shutil
import sys


def allocated_bytes(root):
    """Count hard links once, skip symlinks, tolerate concurrent file removal."""
    total = 0
    seen = set()
    pending = [root]
    while pending:
        directory = pending.pop()
        try:
            with os.scandir(directory) as entries:
                for entry in entries:
                    try:
                        if entry.is_symlink():
                            continue
                        if entry.is_dir(follow_symlinks=False):
                            pending.append(entry.path)
                        elif entry.is_file(follow_symlinks=False):
                            stat = entry.stat(follow_symlinks=False)
                            identity = (stat.st_dev, stat.st_ino)
                            if stat.st_nlink > 1:
                                if identity in seen:
                                    continue
                                seen.add(identity)
                            allocated = stat.st_blocks * 512 if hasattr(stat, 'st_blocks') else stat.st_size
                            total += allocated
                    except FileNotFoundError:
                        pass
        except FileNotFoundError:
            pass
    return total


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', type=Path)
    parser.add_argument('--max-gib', type=float, default=100)
    parser.add_argument('--min-free-gib', type=float, default=30)
    args = parser.parse_args()
    if args.max_gib <= 0 or args.min_free_gib < 0:
        parser.error('budgets must be positive (minimum free may be zero)')
    root = Path(__file__).resolve().parent.parent
    target = args.target_dir or Path(os.environ.get('CARGO_TARGET_DIR', root / 'target'))
    target = target.resolve()
    volume = target
    while not volume.exists():
        volume = volume.parent
    gib = 1024 ** 3
    used = allocated_bytes(target) / gib
    free = shutil.disk_usage(volume).free / gib
    print(f'{target}: {used:.2f} GiB allocated; volume free: {free:.2f} GiB')
    print('Hard links counted once; filesystem clones/snapshots may share blocks.')
    if used > args.max_gib or free < args.min_free_gib:
        print('BUILD STORAGE BUDGET EXCEEDED: stop builds and review obsolete output. No files deleted.')
        return 1
    print('Build storage within budget. No files deleted.')
    return 0


if __name__ == '__main__':
    try:
        sys.exit(main())
    except OSError as error:
        print(f'Cannot measure build storage: {error}', file=sys.stderr)
        sys.exit(2)
