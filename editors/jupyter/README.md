# AdarshLang for Jupyter

Write AdarshLang in Jupyter notebooks, which works well for classrooms. Variables and functions carry over between cells, `dikhao` output appears under each cell, and `kachhua` drawings show up as images.

## Install

1. Install `adarsh` (see the main README) so it is on your PATH, or set the `ADARSH` environment variable to the executable.
2. Install the kernel:

```bash
pip install ./editors/jupyter
python -m adarsh_kernel.install --user
```

3. Start Jupyter (`jupyter lab` or `jupyter notebook`) and pick the **AdarshLang** kernel.

## How it works

The kernel starts one `adarsh kernel-repl` process per notebook and sends each cell to it as a line of JSON. The value of a cell's last expression is shown as the cell's result, the same way the REPL does it.
