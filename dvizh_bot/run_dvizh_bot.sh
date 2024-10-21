#!/bin/bash

# Define the service name
SERVICE_NAME="dvizh_bot.service"

# Check if the service exists
if ! systemctl list-units --full --all | grep -Fq "$SERVICE_NAME"; then
    echo "Service $SERVICE_NAME does not exist. Please create the service file."
    exit 1
fi

# Stop the service if it's running
if systemctl is-active --quiet "$SERVICE_NAME"; then
    echo "Stopping $SERVICE_NAME..."
    sudo systemctl stop "$SERVICE_NAME" || { echo "Failed to stop service."; exit 1; }
fi

# Navigate to the project directory
cd ~/DvizhBot/dvizh_bot/ || { echo "Project directory not found"; exit 1; }

# Build the Rust project
echo "Building the project..."
cargo build --release || { echo "Build failed."; exit 1; }

# Start the service
echo "Starting $SERVICE_NAME..."
sudo systemctl start "$SERVICE_NAME" || { echo "Failed to start service."; exit 1; }

echo "Dvizh Bot is now running."
