"""Register the AdarshLang kernel with Jupyter: python -m adarsh_kernel.install [--user|--sys-prefix]"""

import argparse
import json
import os
import sys
import tempfile

from jupyter_client.kernelspec import KernelSpecManager


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--user", action="store_true", help="install for this user only")
    ap.add_argument("--sys-prefix", action="store_true", help="install into the current Python environment")
    ap.add_argument("--prefix", help="install under this prefix")
    args = ap.parse_args(argv)
    spec = {
        "argv": [sys.executable, "-m", "adarsh_kernel", "-f", "{connection_file}"],
        "display_name": "AdarshLang",
        "language": "adarshlang",
    }
    with tempfile.TemporaryDirectory() as d:
        with open(os.path.join(d, "kernel.json"), "w") as f:
            json.dump(spec, f, indent=2)
        prefix = sys.prefix if args.sys_prefix else args.prefix
        where = KernelSpecManager().install_kernel_spec(d, "adarshlang", user=args.user, prefix=prefix)
    print(f"AdarshLang kernel installed in {where}")


if __name__ == "__main__":
    main()
