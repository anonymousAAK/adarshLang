from flask import Flask, jsonify, render_template, request
import sys
from io import StringIO

from adarsh_lang.home_content import (
    FEATURES,
    GETTING_STARTED_COMMANDS,
    GETTING_STARTED_TIPS,
    README_REFERENCE_URL,
    REPO_URL,
    SAMPLE_PROGRAMS,
    TUTORIAL_TRACKS,
    load_program_source,
)
from adarsh_lang_compiler import (
    AdarshRuntimeError,
    AdarshSemanticError,
    AdarshUserException,
    adarshlang_compile_and_run,
)

app = Flask(__name__)


def build_sample_programs():
    programs = []
    for program in SAMPLE_PROGRAMS:
        programs.append(
            {
                "name": program["name"],
                "description": program["description"],
                "source": load_program_source(program),
            }
        )
    return programs


@app.route("/", methods=["GET"])
def home():
    programs = build_sample_programs()
    default_code = programs[0]["source"] if programs else ""
    return render_template(
        "home.html",
        features=FEATURES,
        getting_started_commands=GETTING_STARTED_COMMANDS,
        getting_started_tips=GETTING_STARTED_TIPS,
        tutorial_tracks=TUTORIAL_TRACKS,
        sample_programs=programs,
        default_code=default_code,
        repo_url=REPO_URL,
        reference_url=README_REFERENCE_URL,
    )


@app.route("/run", methods=["POST"])
def run_code():
    payload = request.get_json(silent=True) or request.form
    source_code = payload.get("source_code", "") if payload else ""

    old_stdout = sys.stdout
    buffer = StringIO()
    sys.stdout = buffer

    error_message = None
    try:
        adarshlang_compile_and_run(source_code)
    except (AdarshRuntimeError, AdarshUserException, AdarshSemanticError) as err:
        error_message = f"Error: {err}"
    except Exception as err:  # pragma: no cover - defensive for unexpected issues
        error_message = f"Unexpected error: {err}"
    finally:
        sys.stdout = old_stdout

    output = buffer.getvalue()
    success = error_message is None

    return jsonify({
        "success": success,
        "output": output,
        "error": error_message,
    })


if __name__ == "__main__":
    app.run(debug=True)
