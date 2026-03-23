FROM rust:1.80 as builder

WORKDIR /usr/src/gws
COPY . .
RUN cargo build --release

FROM python:3.11-slim

WORKDIR /app

# Install OS dependencies if needed, and clean up
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Install python requirements
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Copy binary from builder
COPY --from=builder /usr/src/gws/target/release/gws /usr/local/bin/gws

# Copy application files
COPY app.py .
COPY Agent.md .

# Create configuration directory for gws
RUN mkdir -p /root/.config/gws

EXPOSE 7860
CMD ["uvicorn", "app:app", "--host", "0.0.0.0", "--port", "7860"]
