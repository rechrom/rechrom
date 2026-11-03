#!/usr/bin/env python3
"""Freeze real pages into deterministic, script-free local rendering inputs."""

from __future__ import annotations

import argparse
import hashlib
import json
import mimetypes
import re
from pathlib import Path
from urllib.parse import unquote, urljoin, urlsplit, urlunsplit

import requests


SCRIPT_RE = re.compile(r"<script\b[^>]*>.*?</script\s*>", re.I | re.S)
STYLE_BLOCK_RE = re.compile(r"(<style\b[^>]*>)(.*?)(</style\s*>)", re.I | re.S)
RESOURCE_TAG_RE = re.compile(
    r"<(?:link|img|source|video|audio|input|object|embed|image|use)\b[^>]*>", re.I | re.S
)
ATTRIBUTE_RE = re.compile(
    r"(?P<prefix>\b(?:src|href|poster|data)\s*=\s*)(?P<quote>['\"])(?P<url>.*?)(?P=quote)",
    re.I | re.S,
)
SRCSET_RE = re.compile(
    r"(?P<prefix>\bsrcset\s*=\s*)(?P<quote>['\"])(?P<value>.*?)(?P=quote)",
    re.I | re.S,
)
STYLE_ATTRIBUTE_RE = re.compile(
    r"(?P<prefix>\bstyle\s*=\s*)(?P<quote>['\"])(?P<value>.*?)(?P=quote)",
    re.I | re.S,
)
CSS_URL_RE = re.compile(r"url\(\s*(?P<quote>['\"]?)(?P<url>.*?)(?P=quote)\s*\)", re.I | re.S)
CSS_IMPORT_RE = re.compile(
    r"(?P<prefix>@import\s+)(?P<quote>['\"])(?P<url>.*?)(?P=quote)", re.I | re.S
)


def clean_url(url: str) -> str:
    split = urlsplit(url)
    return urlunsplit((split.scheme, split.netloc, split.path, split.query, ""))


def is_fetchable(url: str) -> bool:
    return urlsplit(url).scheme.lower() in {"http", "https"}


def safe_segment(value: str) -> str:
    value = re.sub(r"[^A-Za-z0-9._-]+", "_", value)
    return value or "_"


class SiteMirror:
    def __init__(self, site_id: str, root: Path, max_resources: int, max_bytes: int):
        self.site_id = site_id
        self.root = root / site_id
        self.root.mkdir(parents=True, exist_ok=True)
        self.max_resources = max_resources
        self.max_bytes = max_bytes
        self.session = requests.Session()
        self.session.headers.update(
            {
                "User-Agent": "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) "
                "AppleWebKit/537.36 (KHTML, like Gecko) Chrome/154.0.0.0 Safari/537.36",
                "Accept-Language": "en-US,en;q=0.9",
            }
        )
        self.resources: dict[str, str] = {}
        self.failures: list[dict[str, str]] = []

    def request(self, url: str) -> requests.Response:
        response = self.session.get(url, timeout=20, allow_redirects=True)
        response.raise_for_status()
        if len(response.content) > self.max_bytes:
            raise RuntimeError(f"resource exceeds {self.max_bytes} bytes")
        return response

    def relative_path(self, url: str, content_type: str) -> Path:
        split = urlsplit(url)
        host = safe_segment(split.netloc.lower())
        parts = [safe_segment(unquote(part)) for part in split.path.split("/") if part]
        if not parts or split.path.endswith("/"):
            parts.append("index")
        path = Path(host, *parts)
        suffix = path.suffix
        mime = content_type.split(";", 1)[0].strip().lower()
        extension = mimetypes.guess_extension(mime) or ""
        if mime == "text/css":
            extension = ".css"
        elif mime in {"application/javascript", "text/javascript"}:
            extension = ".js"
        if not suffix and extension:
            path = path.with_name(path.name + extension)
        if split.query:
            digest = hashlib.sha256(split.query.encode()).hexdigest()[:12]
            path = path.with_name(f"{path.stem}__q_{digest}{path.suffix}")
        return path

    def local_url(self, relative: Path) -> str:
        return "/" + "/".join([self.site_id, *relative.parts])

    def mirror_reference(self, raw: str, base_url: str) -> str:
        raw = raw.strip()
        if not raw or raw.startswith(('#', 'data:', 'blob:', 'about:', 'javascript:')):
            return raw
        absolute = clean_url(urljoin(base_url, raw))
        if not is_fetchable(absolute):
            return raw
        if absolute in self.resources:
            return self.local_url(Path(self.resources[absolute]))
        if len(self.resources) >= self.max_resources:
            self.failures.append({"url": absolute, "error": "resource limit reached"})
            return raw
        try:
            response = self.request(absolute)
            content_type = response.headers.get("Content-Type", "application/octet-stream")
            relative = self.relative_path(absolute, content_type)
            self.resources[absolute] = relative.as_posix()
            body = response.content
            mime = content_type.split(";", 1)[0].strip().lower()
            if mime == "text/css" or relative.suffix.lower() == ".css":
                encoding = response.encoding or "utf-8"
                css = body.decode(encoding, "replace")
                body = self.rewrite_css(css, response.url).encode("utf-8")
            target = self.root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(body)
            return self.local_url(relative)
        except Exception as error:
            self.failures.append({"url": absolute, "error": repr(error)})
            return raw

    def rewrite_css(self, css: str, base_url: str) -> str:
        def imported(match: re.Match[str]) -> str:
            local = self.mirror_reference(match.group("url"), base_url)
            return f"{match.group('prefix')}{match.group('quote')}{local}{match.group('quote')}"

        css = CSS_IMPORT_RE.sub(imported, css)

        def resource(match: re.Match[str]) -> str:
            local = self.mirror_reference(match.group("url"), base_url)
            quote = match.group("quote")
            return f"url({quote}{local}{quote})"

        return CSS_URL_RE.sub(resource, css)

    def rewrite_html(self, html: str, base_url: str) -> str:
        html = SCRIPT_RE.sub("", html)

        def style_block(match: re.Match[str]) -> str:
            return match.group(1) + self.rewrite_css(match.group(2), base_url) + match.group(3)

        html = STYLE_BLOCK_RE.sub(style_block, html)

        def resource_tag(match: re.Match[str]) -> str:
            tag = match.group(0)

            def attribute(attribute_match: re.Match[str]) -> str:
                local = self.mirror_reference(attribute_match.group("url"), base_url)
                quote = attribute_match.group("quote")
                return f"{attribute_match.group('prefix')}{quote}{local}{quote}"

            tag = ATTRIBUTE_RE.sub(attribute, tag)

            def srcset(srcset_match: re.Match[str]) -> str:
                rewritten = []
                for candidate in srcset_match.group("value").split(","):
                    fields = candidate.strip().split()
                    if not fields:
                        continue
                    fields[0] = self.mirror_reference(fields[0], base_url)
                    rewritten.append(" ".join(fields))
                quote = srcset_match.group("quote")
                return f"{srcset_match.group('prefix')}{quote}{', '.join(rewritten)}{quote}"

            return SRCSET_RE.sub(srcset, tag)

        html = RESOURCE_TAG_RE.sub(resource_tag, html)

        def style_attribute(match: re.Match[str]) -> str:
            css = self.rewrite_css(match.group("value"), base_url)
            quote = match.group("quote")
            return f"{match.group('prefix')}{quote}{css}{quote}"

        return STYLE_ATTRIBUTE_RE.sub(style_attribute, html)

    def mirror_page(self, url: str) -> dict:
        response = self.request(url)
        response.encoding = response.encoding or "utf-8"
        html = self.rewrite_html(response.text, response.url)
        entry = self.root / "index.html"
        entry.write_text(html)
        metadata = {
            "id": self.site_id,
            "source_url": url,
            "final_url": response.url,
            "entry": entry.name,
            "resources": len(self.resources),
            "failures": self.failures,
        }
        (self.root / "snapshot.json").write_text(json.dumps(metadata, indent=2))
        return metadata


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--base-url", default="http://127.0.0.1:9877")
    parser.add_argument("--max-resources", type=int, default=250)
    parser.add_argument("--max-bytes", type=int, default=12 * 1024 * 1024)
    args = parser.parse_args()
    sites = json.loads(args.manifest.read_text())
    args.output.mkdir(parents=True, exist_ok=True)
    local_manifest = []
    results = []
    for site in sites:
        mirror = SiteMirror(site["id"], args.output, args.max_resources, args.max_bytes)
        row = {"id": site["id"], "url": site["url"], "error": None}
        try:
            row.update(mirror.mirror_page(site["url"]))
            local_manifest.append(
                {
                    "id": site["id"],
                    "url": f"{args.base_url.rstrip('/')}/{site['id']}/index.html",
                    "scrolls": site["scrolls"],
                }
            )
        except Exception as error:
            row["error"] = repr(error)
        results.append(row)
        (args.output / "snapshot_results.json").write_text(json.dumps(results, indent=2))
        print(json.dumps({"id": site["id"], "error": row["error"], "resources": len(mirror.resources)}), flush=True)
    (args.output / "sites.json").write_text(json.dumps(local_manifest, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
