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
    echo "Iteration #$counter: Running pi command..."
    pi code -p "Read PROMPT.md and follow the instructions"

    # Check if the progress file exists
    if [[ -f "$PROGRESS_FILE" ]]; then
        # Check for the completion string
        if grep -q "$TARGET_STRING" "$PROGRESS_FILE"; then
            echo "Success: Completion string found in $PROGRESS_FILE."
            pi code -p "Summarize the progress report in progress.md and the status of tasks in tasks.md and the learnings in memories.md"
            exit 0
    fi

    # Increment counter
    ((counter++))
done