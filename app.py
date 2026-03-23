from fastapi import FastAPI, HTTPException, Request
from fastapi.responses import RedirectResponse
from pydantic import BaseModel
import subprocess
import os
import json
import shlex
import re

app = FastAPI(
    title="GWS CLI API",
    description="API wrapper for Google Workspace CLI",
    version="1.0.0",
    docs_url="/api-docs" # Map the Swagger UI to /api-docs to meet requirement
)

class ExecuteRequest(BaseModel):
    command: str

@app.get("/health")
def health_check():
    """Returns HTTP 200 when the app is ready."""
    return {"status": "ok"}

@app.post("/execute")
def execute_command(req: ExecuteRequest):
    """
    Execute a CLI command.
    Example command: "drive files list --params '{\"pageSize\": 5}'"
    """
    # Provide the token if available
    env = os.environ.copy()

    try:
        # Secure execution: use shlex.split to parse the command string into a list of arguments,
        # avoiding shell=True which is vulnerable to command injection.
        args = shlex.split(req.command)
        full_command = ["gws"] + args

        result = subprocess.run(
            full_command,
            capture_output=True,
            text=True,
            env=env
        )

        # Try to parse the output as JSON if possible, since GWS outputs structured JSON
        output = result.stdout
        try:
            output = json.loads(result.stdout)
        except json.JSONDecodeError:
            pass

        return {
            "exit_code": result.returncode,
            "output": output,
            "error": result.stderr
        }
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/oauth/login")
def oauth_login():
    """
    Provides an endpoint to initiate the Google OAuth login.
    This runs 'gws auth login', captures the URL it generates, and redirects the user to it.
    """
    env = os.environ.copy()
    process = None
    try:
        process = subprocess.Popen(
            ["gws", "auth", "login"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env
        )

        url = None
        for _ in range(30):
            line = process.stderr.readline()
            if not line:
                break

            # Look for the URL pattern inside the output
            match = re.search(r'(https://accounts\.google\.com/[^\s]+)', line)
            if match:
                url = match.group(1)
                break

        if url:
            # We redirect to the URL so the space embedding opens it.
            if process:
                process.terminate()
            return RedirectResponse(url=url)
        else:
            if process:
                process.terminate()
            return {"message": "Failed to extract OAuth URL. Ensure OAuth client is configured.", "details": "You can configure it by running 'gws auth setup' or setting GOOGLE_WORKSPACE_CLI_CLIENT_ID and GOOGLE_WORKSPACE_CLI_CLIENT_SECRET. Alternatively, execute 'auth login' via the /execute endpoint."}

    except Exception as e:
        if process:
            process.terminate()
        raise HTTPException(status_code=500, detail=str(e))

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=7860)
