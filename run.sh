#!/bin/bash

# Configuration
PROGRESS_FILE="progress.md"
TARGET_STRING="<promise>COMPLETE</promise>"
PI_COMMAND='pi code -p "Read PROMPT.md and follow the instructions"'

# Initialize counter
counter=1

echo "Starting task runner..."

while true; do
    # Execute the pi command
    echo "------------------------------------------"
    echo "Attempt #$counter: Running pi command..."
    pi code -p "Read PROMPT.md and follow the instructions"

    # Check if the progress file exists
    if [[ -f "$PROGRESS_FILE" ]]; then
        # Check for the completion string
        if grep -q "$TARGET_STRING" "$PROGRESS_FILE"; then
            echo "Success: Completion string found in $PROGRESS_FILE."
            exit 0
        else
            echo "Task not yet complete"
        fi
    fi
    
    # Increment counter
    ((counter++))
done