#!/bin/bash

# Script to rename scenario folders to PascalCase and update references

# Define the mapping of old names to new names
declare -A name_map=(
    ["Circle Experiment"]="CircleExperiment"
    ["Collaborative Complex"]="CollaborativeComplex"
    ["Collaborative GP"]="CollaborativeGP"
    ["Communications Failure Experiment"]="CommunicationsFailureExperiment"
    ["Environment Obstacles Experiment"]="EnvironmentObstaclesExperiment"
    ["Iteration Amount Experiment"]="IterationAmountExperiment"
    ["Junction Experiment"]="JunctionExperiment"
    ["Junction Twoway"]="JunctionTwoway"
    ["Obstacle Shapes Showcase"]="ObstacleShapesShowcase"
    ["Schedules Experiment"]="SchedulesExperiment"
    ["Solo GP"]="SoloGP"
    ["Structured Junction"]="StructuredJunction"
    ["Structured Junction Twoway"]="StructuredJunctionTwoway"
    ["Tracking Factor Showcase"]="TrackingFactorShowcase"
    ["Varying Network Connectivity Experiment"]="VaryingNetworkConnectivityExperiment"
)

# Base directory for scenarios
SCENARIOS_DIR="config/scenarios"

# Function to update references in a file
update_references() {
    local file=$1
    
    # Skip if file doesn't exist
    if [ ! -f "$file" ]; then
        return
    fi
    
    echo "Updating references in $file"
    
    # Update all references in the file
    for old_name in "${!name_map[@]}"; do
        new_name=${name_map[$old_name]}
        # Escape spaces and other special characters for sed
        escaped_old_name=$(echo "$old_name" | sed 's/[\/&]/\\&/g')
        escaped_new_name=$(echo "$new_name" | sed 's/[\/&]/\\&/g')
        
        # Update references in the file
        sed -i "s/$escaped_old_name/$escaped_new_name/g" "$file"
    done
}

# First, update all references in config files
echo "Updating references in config files..."
for old_name in "${!name_map[@]}"; do
    config_file="$SCENARIOS_DIR/$old_name/config.toml"
    update_references "$config_file"
done

# Update references in other important files
update_references "crates/magics/src/simulation_loader.rs"
update_references "crates/magics/src/main.rs"
update_references "python_api/example.py"
update_references "python_api/magics_client.py"

# Now rename the directories
echo "Renaming directories..."
for old_name in "${!name_map[@]}"; do
    new_name=${name_map[$old_name]}
    old_path="$SCENARIOS_DIR/$old_name"
    new_path="$SCENARIOS_DIR/$new_name"
    
    # Skip if old directory doesn't exist or new directory already exists
    if [ ! -d "$old_path" ] || [ -d "$new_path" ]; then
        echo "Skipping $old_name -> $new_name (directory issue)"
        continue
    fi
    
    echo "Renaming $old_path -> $new_path"
    mv "$old_path" "$new_path"
done

echo "Done! All scenario folders have been renamed to PascalCase."
