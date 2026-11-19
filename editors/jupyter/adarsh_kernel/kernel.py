"""AdarshLang Jupyter kernel.

Each notebook gets one `adarsh kernel-repl` process, so variables and
functions defined in one cell are available in later cells. Output from
`dikhao` is shown as text, errors in red, the value of a cell's last
expression as its result, and kachhua (turtle) drawings as SVG images.
"""

import json
import os
import shutil
import subprocess

from ipykernel.kernelbase import Kernel

from . import __version__


def find_adarsh():
    return os.environ.get("ADARSH") or shutil.which("adarsh") or "adarsh"


class AdarshKernel(Kernel):
    implementation = "adarsh_kernel"
    implementation_version = __version__
    language = "adarshlang"
    language_version = "0.3"
    language_info = {
        "name": "adarshlang",
        "mimetype": "text/x-adarshlang",
        "file_extension": ".aak",
        "codemirror_mode": "javascript",
        "pygments_lexer": "javascript",
    }
    banner = "AdarshLang: Hinglish programming language"

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self.proc = None

    def _start(self):
        if self.proc is None or self.proc.poll() is not None:
            self.proc = subprocess.Popen(
                [find_adarsh(), "kernel-repl"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                text=True,
                encoding="utf-8",
                bufsize=1,
            )

    def do_execute(self, code, silent, store_history=True, user_expressions=None, allow_stdin=False):
        if not code.strip():
            return {"status": "ok", "execution_count": self.execution_count, "payload": [], "user_expressions": {}}
        try:
            self._start()
            self.proc.stdin.write(json.dumps({"code": code}) + "\n")
            self.proc.stdin.flush()
            line = self.proc.stdout.readline()
            if not line:
                raise RuntimeError("adarsh band ho gaya. Kernel restart karo.")
            reply = json.loads(line)
        except (OSError, RuntimeError, ValueError) as e:
            self.send_response(self.iopub_socket, "stream", {"name": "stderr", "text": f"{e}\n"})
            return {"status": "error", "ename": "KernelError", "evalue": str(e), "traceback": [str(e)],
                    "execution_count": self.execution_count}
        if not silent:
            if reply.get("output"):
                self.send_response(self.iopub_socket, "stream", {"name": "stdout", "text": reply["output"]})
            if reply.get("svg"):
                self.send_response(self.iopub_socket, "display_data",
                                   {"data": {"image/svg+xml": reply["svg"]}, "metadata": {}})
            if reply.get("value") is not None:
                self.send_response(self.iopub_socket, "execute_result",
                                   {"execution_count": self.execution_count,
                                    "data": {"text/plain": reply["value"]}, "metadata": {}})
        if reply.get("error"):
            err = reply["error"]
            if not silent:
                self.send_response(self.iopub_socket, "stream", {"name": "stderr", "text": err + "\n"})
            first = err.split("\n", 1)[0]
            return {"status": "error", "ename": first.split(":", 1)[0], "evalue": first, "traceback": err.split("\n"),
                    "execution_count": self.execution_count}
        return {"status": "ok", "execution_count": self.execution_count, "payload": [], "user_expressions": {}}

    def do_shutdown(self, restart):
        if self.proc is not None:
            self.proc.terminate()
            self.proc = None
        return {"status": "ok", "restart": restart}
