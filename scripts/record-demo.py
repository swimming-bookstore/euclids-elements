#!/usr/bin/env python3
"""Record docs/demo_{book}_{slug}.mp4 of a proposition via ~/demo."""

from __future__ import annotations

import argparse
import http.server
import os
import socket
import threading
from functools import partial
from pathlib import Path

from record.session import RecordSession

ROOT = Path(__file__).resolve().parent.parent / "dist"
DOCS = Path(__file__).resolve().parent.parent / "docs"


def free_port() -> int:
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    p = s.getsockname()[1]
    s.close()
    return p


def serve(port: int) -> http.server.HTTPServer:
    handler = partial(http.server.SimpleHTTPRequestHandler, directory=str(ROOT))
    httpd = http.server.HTTPServer(("127.0.0.1", port), handler)
    threading.Thread(target=httpd.serve_forever, daemon=True).start()
    return httpd


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--book", type=int, default=1)
    parser.add_argument("--prop", type=int, default=1)
    parser.add_argument("--slug", default="proposition1")
    parser.add_argument("--hold", type=float, default=182)
    args = parser.parse_args()

    os.environ.setdefault("DISPLAY", ":0.0")
    DOCS.mkdir(exist_ok=True)
    out = DOCS / f"demo_book{args.book}_{args.slug}.mp4"
    title = f"euclid-book{args.book}-{args.slug}"
    port = free_port()
    httpd = serve(port)
    url = (
        f"http://127.0.0.1:{port}/index.html"
        f"?record=1&autoplay=1#/{args.book}/{args.prop}"
    )
    rec = RecordSession(
        url=url,
        out=out,
        title=title,
        width=900,
        height=800,
        fps=15,
        max_sec=args.hold + 20,
        cdp_port=9331,
    )
    rec.attach_chrome()
    try:
        rec.wait_js(
            "!!document.querySelector('.stage.record[data-ready=\"1\"] .figure-frame')",
            25,
            "ready",
        )
        rec.start_capture()
        rec.hold(args.hold)
        rec._stop.set()
        if rec._cap:
            rec._cap.join(timeout=15)
        ff = rec._ff
        if ff and ff.stdin:
            try:
                ff.stdin.close()
            except OSError:
                pass
        if ff:
            try:
                ff.wait(timeout=120)
            except Exception:
                ff.send_signal(__import__("signal").SIGINT)
                try:
                    ff.wait(timeout=30)
                except Exception:
                    pass
    finally:
        rec.__exit__(None, None, None)
        httpd.shutdown()
    print(out)


if __name__ == "__main__":
    main()
