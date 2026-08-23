#!/usr/bin/env python3
"""Stub DeepSeek endpoint for dba/04-tools/implementer/tests/codeos-implement-tests.sh.

Stands in for the chat/completions API so the delimited output protocol, the nonce round trip, and
every fail-closed path can be tested without network access or API spend.

It reads the posted request body, recovers the `output_nonce` the tool generated for that run, and
renders the fixture named by CODEOS_STUB_FIXTURE with {N} substituted by that nonce. That means the
stub responds the way a compliant model would, rather than replaying a canned nonce the tool would
reject.

Env:
  CODEOS_STUB_FIXTURE  path to a fixture file; its text becomes the message content, after {N} ->
                       the request's own nonce. Use {{ and }} for literal braces.
  CODEOS_STUB_PORT     port to bind (default 8931)
  CODEOS_STUB_STATUS   HTTP status to return (default 200) — set non-2xx to exercise exit 8
  CODEOS_STUB_RAW      if set, return this literal body instead of a chat-completions envelope
  CODEOS_STUB_FINISH_REASON
                       completion finish reason (default stop)
  CODEOS_STUB_SHAPE    "deepseek" (default) or "gemini". Selects the returned model id and the usage
                       shape. The gemini shape carries prompt/completion/total only — no reasoning
                       field and no cache fields — because that is what the real endpoint returns,
                       and the adapter's derived accounting has to be tested against it.
  CODEOS_STUB_REQUEST_DUMP
                       optional path; the raw posted request body is written here so a test can
                       assert on what the adapter actually sent (e.g. that `thinking` is absent).
"""
import http.server
import json
import os
import re
import sys

FIXTURE = os.environ.get("CODEOS_STUB_FIXTURE", "")
PORT = int(os.environ.get("CODEOS_STUB_PORT", "8931"))
STATUS = int(os.environ.get("CODEOS_STUB_STATUS", "200"))
RAW = os.environ.get("CODEOS_STUB_RAW")
FINISH_REASON = os.environ.get("CODEOS_STUB_FINISH_REASON", "stop")
SHAPE = os.environ.get("CODEOS_STUB_SHAPE", "deepseek")
REQUEST_DUMP = os.environ.get("CODEOS_STUB_REQUEST_DUMP")

# Two providers, two usage shapes. DeepSeek's completion_tokens INCLUDES its reported reasoning;
# Gemini returns three fields only, and its completion_tokens EXCLUDES the residual.
USAGE = {
    "deepseek": (
        "deepseek-v4-flash",
        {
            "prompt_tokens": 1234,
            "completion_tokens": 567,
            "total_tokens": 1801,
            "prompt_cache_hit_tokens": 1000,
            "prompt_cache_miss_tokens": 234,
            "completion_tokens_details": {"reasoning_tokens": 321},
        },
    ),
    "gemini": (
        "gemini-3.7-flash",
        {"prompt_tokens": 1234, "completion_tokens": 567, "total_tokens": 2101},
    ),
}


class Handler(http.server.BaseHTTPRequestHandler):
    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8", "replace")
        if REQUEST_DUMP:
            with open(REQUEST_DUMP, "w") as fh:
                fh.write(body)

        # Recover the nonce the tool minted for this run.
        nonce = ""
        try:
            payload = json.loads(body)
            user = "".join(
                m.get("content", "")
                for m in payload.get("messages", [])
                if m.get("role") == "user"
            )
            m = re.search(r"output_nonce:\s*([0-9a-f]+)", user)
            if m:
                nonce = m.group(1)
        except Exception:
            pass

        if RAW is not None:
            content = RAW
        elif FIXTURE:
            with open(FIXTURE, "r") as fh:
                content = fh.read()
            content = content.replace("{{", "\x00").replace("}}", "\x01")
            content = content.replace("{N}", nonce)
            content = content.replace("\x00", "{").replace("\x01", "}")
        else:
            content = ""

        model_id, usage = USAGE[SHAPE]
        out = json.dumps(
            {
                "choices": [
                    {
                        "finish_reason": FINISH_REASON,
                        "message": {"content": content},
                    }
                ],
                "model": model_id,
                "usage": usage,
            }
        ).encode()

        self.send_response(STATUS)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(out)))
        self.end_headers()
        self.wfile.write(out)

    def log_message(self, *args):
        pass


if __name__ == "__main__":
    srv = http.server.HTTPServer(("127.0.0.1", PORT), Handler)
    sys.stderr.write(f"stub listening on {PORT}\n")
    sys.stderr.flush()
    srv.serve_forever()
