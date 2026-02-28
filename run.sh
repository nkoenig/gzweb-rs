#!/bin/bash
# Check if port 8000 is in use and kill the process
if lsof -i :8000 -t >/dev/null; then
    echo "Port 8000 is in use. Killing the process..."
    kill $(lsof -i :8000 -t)
    sleep 1
fi

echo "Starting server at http://localhost:8000"
python3 -m http.server 8000
