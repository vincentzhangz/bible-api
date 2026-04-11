#!/bin/bash
set -e

echo "Running database migrations..."
bible-api migrate

echo "Ingesting data..."
bible-api ingest

echo "Starting API server..."
exec bible-api serve
