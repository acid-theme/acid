#!/usr/bin/env python3
"""Publish each port from the hub to its own repository in the organisation.

The hub is the only repository anyone edits. Port repositories are mirrors: this
script renders each one's contents from `resources/ports.toml` and pushes a
single commit per change. It is idempotent — a port whose files have not changed
produces no commit and no push.

Python with `tomllib` rather than a crate in the workspace, because the work is
file copying and git plumbing, and this way CI needs nothing built to run it.

    publish.py --check            validate the registry, touch no network
    publish.py --readmes          write each port's README in this repository
    publish.py --dry-run          build the trees, report what would change
    publish.py --out DIR          build the trees into DIR and keep them
    publish.py                    push every port that changed
    publish.py --only nvim        just one port
    publish.py --tag v0.1.0       also move that tag in each mirror

Set ACID_PUBLISH_TOKEN to push with a token; without it, git's own credentials
are used, which is what you want when running this by hand.
"""

from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys
import tempfile
import json
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "resources" / "ports.toml"
README_TEMPLATE = ROOT / "resources" / "port-readme.md"
PALETTE = ROOT / "palette.json"
LICENSE = ROOT / "LICENSE"


class Failure(Exception):
    """Something the user has to fix."""


def run(args: list[str], cwd: Path | None = None, check: bool = True) -> str:
    result = subprocess.run(
        args, cwd=cwd, capture_output=True, text=True, check=False
    )
    if check and result.returncode != 0:
        raise Failure(
            f"{' '.join(args)} failed ({result.returncode}):\n"
            f"{result.stderr.strip() or result.stdout.strip()}"
        )
    return result.stdout.strip()


def load_registry() -> dict:
    with REGISTRY.open("rb") as handle:
        registry = tomllib.load(handle)
    for key in ("org", "hub"):
        if key not in registry:
            raise Failure(f"{REGISTRY.name}: missing `{key}`")
    if not registry.get("port"):
        raise Failure(f"{REGISTRY.name}: no ports declared")
    return registry


def validate(registry: dict) -> list[str]:
    """Check the registry against the working tree. Returns the problems found."""
    problems: list[str] = []
    seen: set[str] = set()
    declared_templates: set[Path] = set()

    for port in registry["port"]:
        label = port.get("name", "<unnamed>")
        for key in ("name", "title", "description", "template", "install", "publish"):
            if not port.get(key):
                problems.append(f"{label}: missing `{key}`")
        if port.get("name") in seen:
            problems.append(f"{label}: duplicate name")
        seen.add(port.get("name"))

        template = port.get("template")
        if template:
            path = ROOT / template
            declared_templates.add(path)
            if not path.is_file():
                problems.append(f"{label}: template {template} does not exist")

        for rule in port.get("publish", []):
            src = rule.get("src")
            if src is None or "dest" not in rule:
                problems.append(f"{label}: publish rule needs `src` and `dest`")
                continue
            path = ROOT / src
            if not path.is_dir():
                problems.append(f"{label}: publish src {src} is not a directory")
            elif not any(p.is_file() for p in path.iterdir()):
                problems.append(f"{label}: publish src {src} holds no files")
            dest = rule["dest"]
            if dest.startswith("/") or ".." in Path(dest).parts:
                problems.append(f"{label}: publish dest {dest!r} must stay inside the repo")

        readme = readme_path(port)
        if not readme.is_file():
            problems.append(f"{label}: {readme.relative_to(ROOT)} is missing; run `make docs`")
        elif port.get("install") and port.get("title"):
            files = [
                str((Path(rule["dest"]) / item.name) if rule["dest"] else Path(item.name))
                for rule in port.get("publish", [])
                if (ROOT / rule["src"]).is_dir()
                for item in sorted((ROOT / rule["src"]).iterdir())
                if item.is_file()
            ]
            if readme.read_text() != render_readme(registry, port, files):
                problems.append(
                    f"{label}: {readme.relative_to(ROOT)} is out of date; run `make docs`"
                )

    # Every port template in the tree must be registered, or it ships nowhere.
    for path in sorted(ROOT.glob("ports/*/*.tera")):
        if path not in declared_templates:
            problems.append(
                f"{path.relative_to(ROOT)} has no entry in {REGISTRY.name}"
            )

    return problems


def port_dir(port: dict) -> Path:
    return ROOT / Path(port["template"]).parent


def readme_path(port: dict) -> Path:
    return port_dir(port) / "README.md"


def flavour_summary() -> str:
    """One line naming each flavour, taken from the palette rather than repeated."""
    with PALETTE.open() as handle:
        palette = json.load(handle)
    parts = [
        f"**{flavor['name']}** (`{flavor['colors']['base']['hex']}`)"
        for flavor in palette.values()
    ]
    return f"Two flavours: {parts[0]}, vibrant, and {parts[1]}, muted."


def render_readme(registry: dict, port: dict, files: list[str]) -> str:
    listing = "\n".join(f"- `{name}`" for name in files)
    text = README_TEMPLATE.read_text()
    for key, value in {
        "%%TITLE%%": port["title"],
        "%%DESCRIPTION%%": port["description"],
        "%%INSTALL%%": port["install"].strip(),
        "%%FILES%%": listing,
        "%%TEMPLATE%%": port["template"],
        "%%HUB%%": registry["hub"],
        "%%NAME%%": port["name"],
        "%%FLAVOURS%%": flavour_summary(),
    }.items():
        text = text.replace(key, value)
    return text


def build_tree(registry: dict, port: dict, into: Path) -> list[str]:
    """Lay out the mirror's contents. Returns the published file paths."""
    published: list[str] = []
    for rule in port["publish"]:
        source = ROOT / rule["src"]
        target = into / rule["dest"] if rule["dest"] else into
        target.mkdir(parents=True, exist_ok=True)
        for item in sorted(source.iterdir()):
            if not item.is_file():
                continue
            shutil.copy2(item, target / item.name)
            published.append(str((target / item.name).relative_to(into)))

    shutil.copy2(readme_path(port), into / "README.md")
    shutil.copy2(LICENSE, into / "LICENSE")
    return published


def clear_tracked(repo: Path) -> None:
    """Empty the checkout so deletions propagate, keeping .git intact."""
    for item in repo.iterdir():
        if item.name == ".git":
            continue
        shutil.rmtree(item) if item.is_dir() else item.unlink()


def remote_url(registry: dict, port: dict) -> str:
    slug = f"{registry['org']}/{port['name']}"
    token = os.environ.get("ACID_PUBLISH_TOKEN")
    if token:
        return f"https://x-access-token:{token}@github.com/{slug}.git"
    return f"https://github.com/{slug}.git"


def describe_hub() -> str:
    sha = run(["git", "rev-parse", "--short", "HEAD"], cwd=ROOT, check=False)
    return sha or "unknown"


def publish(
    registry: dict, port: dict, *, dry_run: bool, tag: str | None, out: Path | None
) -> bool:
    """Sync one port. Returns whether anything changed."""
    name = port["name"]
    slug = f"{registry['org']}/{name}"

    if out is not None:
        repo = out / name
        if repo.exists():
            shutil.rmtree(repo)
        repo.mkdir(parents=True)
        files = build_tree(registry, port, repo)
        print(f"  {slug} -> {repo}")
        for path in files + ["README.md", "LICENSE"]:
            print(f"      {path}")
        return True

    with tempfile.TemporaryDirectory(prefix=f"acid-{name}-") as workdir:
        repo = Path(workdir) / name

        if dry_run:
            repo.mkdir(parents=True)
            files = build_tree(registry, port, repo)
            print(f"  {slug}")
            for path in files + ["README.md", "LICENSE"]:
                print(f"      {path}")
            return True

        try:
            run(["git", "clone", "--quiet", remote_url(registry, port), str(repo)])
        except Failure as error:
            message = str(error)
            if "not found" in message.lower() or "repository not found" in message.lower():
                raise Failure(
                    f"{slug} does not exist. Port repositories are created once, "
                    f"by hand; this only pushes to them."
                ) from error
            raise

        clear_tracked(repo)
        build_tree(registry, port, repo)

        run(["git", "add", "--all"], cwd=repo)
        if not run(["git", "status", "--porcelain"], cwd=repo):
            print(f"  {slug}: already up to date")
            return False

        subject = f"Sync from {registry['hub']}@{describe_hub()}"
        if tag:
            subject = f"Sync {tag} from {registry['hub']}@{describe_hub()}"
        run(
            [
                "git",
                "-c", "user.name=acid-theme bot",
                "-c", "user.email=noreply@github.com",
                "commit", "--quiet", "--message", subject,
            ],
            cwd=repo,
        )
        head = run(["git", "symbolic-ref", "--short", "HEAD"], cwd=repo, check=False)
        run(["git", "push", "--quiet", "origin", f"HEAD:{head or 'main'}"], cwd=repo)
        if tag:
            run(["git", "tag", "--force", tag], cwd=repo)
            run(["git", "push", "--quiet", "--force", "origin", tag], cwd=repo)
        print(f"  {slug}: pushed {subject}")
        return True


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--check", action="store_true",
                        help="validate the registry and exit")
    parser.add_argument("--readmes", action="store_true",
                        help="write each port's README in this repository and exit")
    parser.add_argument("--dry-run", action="store_true",
                        help="build the trees and report, push nothing")
    parser.add_argument("--only", metavar="NAME", action="append",
                        help="limit to these ports")
    parser.add_argument("--tag", metavar="TAG",
                        help="also move this tag in each mirror")
    parser.add_argument("--out", metavar="DIR", type=Path,
                        help="build the trees into DIR and keep them; pushes nothing")
    args = parser.parse_args()

    try:
        registry = load_registry()

        if args.readmes:
            for port in registry["port"]:
                files = [
                    str((Path(rule["dest"]) / item.name) if rule["dest"] else Path(item.name))
                    for rule in port["publish"]
                    if (ROOT / rule["src"]).is_dir()
                    for item in sorted((ROOT / rule["src"]).iterdir())
                    if item.is_file()
                ]
                target = readme_path(port)
                target.write_text(render_readme(registry, port, files))
                print(f"readme: wrote {target.relative_to(ROOT)}")
            return 0

        problems = validate(registry)
        if problems:
            print(f"{REGISTRY.name} does not match the working tree:", file=sys.stderr)
            for problem in problems:
                print(f"  - {problem}", file=sys.stderr)
            return 1
        if args.check:
            print(f"registry: {len(registry['port'])} ports, all consistent")
            return 0

        ports = registry["port"]
        if args.only:
            wanted = set(args.only)
            unknown = wanted - {p["name"] for p in ports}
            if unknown:
                raise Failure(f"no such port: {', '.join(sorted(unknown))}")
            ports = [p for p in ports if p["name"] in wanted]

        pushing = not (args.dry_run or args.out)
        print(
            f"{'publishing' if pushing else 'would publish'} "
            f"{len(ports)} port(s) to {registry['org']}:"
        )
        changed = sum(
            publish(registry, port, dry_run=args.dry_run, tag=args.tag, out=args.out)
            for port in ports
        )
        if pushing:
            print(f"{changed} of {len(ports)} port(s) changed")
        return 0
    except Failure as error:
        print(f"publish: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
