## 1. Deployment Configuration

The HF space should be able to execute this cli in a terminal environment and expose the usage options via api endpoints to the url of the space, whilst pulling all credentials on hf space loading from the hf space environment secrets and have a google OAUTH feature too opening on the space embedding url

### Target Space
- **Profile:** `harvesthealth`
- **Space:** `harvesthealth/reor`
- **Full Identifier:** `harvesthealth/harvesthealth/reor`
- **Frontend Port:** `7860` (mandatory for all Hugging Face Spaces)

### Deployment Method
Choose the correct SDK based on the app type based on the codebase language:

- **Gradio SDK** — for Gradio applications
- **Streamlit SDK** — for Streamlit applications
- **Docker SDK** — for all other applications (recommended default for flexibility)

### HF Token
- The environment variable **`HF_TOKEN` will always be provided at execution time**.
- Never hardcode the token. Always read it from the environment.
- All monitoring and log‑streaming commands rely on `HF_TOKEN`.

### Required Files
- `Dockerfile` (or `app.py` for Gradio/Streamlit SDKs)
- `README.md` with Hugging Face YAML frontmatter:
  ```yaml
  ---
  title: <APP NAME>
  sdk: docker | gradio | streamlit
  app_port: 7860
  ---
  ```
- `.hfignore` to exclude unnecessary files
- This `Agent.md` file (must be committed before deployment)

---

## 2. API Exposure and Documentation

### Mandatory Endpoints
Every deployment **must** expose:

- **`/health`**
  - Returns HTTP 200 when the app is ready.
  - Required for Hugging Face to transition the Space from *starting* → *running*.

- **`/api-docs`**
  - Documents **all** available API endpoints.
  - Must be reachable at:
    `https://HF_PROFILE-harvesthealth/reor.hf.space/api-docs`

### Functional Endpoints
Document each endpoint here. For every endpoint, include:

- **Method:** GET/POST/PUT/DELETE
- **Path:** `/predict`, `/generate`, `/upload`, etc.
- **Purpose:** What the endpoint does
- **Request Example:** JSON or query parameters
- **Response Example:** JSON schema or example payload

Example format:

```
### /execute
- Method: POST
- Purpose: Execute a CLI command
- Request:
  {
    "command": "drive files list --params '{\"pageSize\": 5}'"
  }
- Response:
  {
    "output": "..."
  }
```

All endpoints listed here **must** appear in `/api-docs`.

---

## 3. Deployment Workflow

### Standard Deployment Command
After any code change, run:

```bash
hf upload harvesthealth/harvesthealth/reor --repo-type=space
---

Scan build and run logs # Get container logs (SSE) curl -N
-H "Authorization: Bearer $HF_TOKEN"
"https://huggingface.co/api/spaces/harvesthealth/harvesthealth/reor/logs/build"

Get build logs (SSE)
curl -N
-H "Authorization: Bearer $HF_TOKEN"
"https://huggingface.co/api/spaces/harvesthealth/harvesthealth/reor/logs/run"

after 300 seconds to see if the deployment has been successful, and if not, fix the errors of deployment, and redeploy and monitor in a cycle until the space is running and reacts to the api endpoints you created.
