#!/bin/bash

# Colors for better output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Detect shell type
SHELL_TYPE=$(basename "$SHELL")

# Check if virtual environment exists
if [ -d ".venv" ]; then
    echo -e "${YELLOW}Virtual environment already exists, skipping creation.${NC}"
else
    echo -e "${BLUE}Creating virtual environment for Magics project...${NC}"
    python3 -m venv .venv
fi

# Activate the virtual environment based on shell type
echo -e "${BLUE}Activating virtual environment...${NC}"
# When running in bash script, we always use the standard activation script
# regardless of the user's shell
if [ "$(uname)" == "Darwin" ] || [ "$(uname)" == "Linux" ]; then
    echo -e "${BLUE}Detected Unix/Linux/MacOS${NC}"
    source .venv/bin/activate
elif [ "$(expr substr $(uname -s) 1 5)" == "MINGW" ] || [ "$(expr substr $(uname -s) 1 10)" == "MSYS_NT-10" ]; then
    echo -e "${BLUE}Detected Windows${NC}"
    source .venv/Scripts/activate
else
    echo "Unsupported operating system. Please activate the virtual environment manually."
    exit 1
fi

# Check if pip is already up to date
PIP_VERSION=$(pip --version)
echo -e "${BLUE}Checking pip version: ${PIP_VERSION}${NC}"
pip install --upgrade pip

# Check if the project is already installed
if pip list | grep -q "magics-gym"; then
    echo -e "${YELLOW}Magics project already installed, updating...${NC}"
    pip install -e . --upgrade
else
    echo -e "${BLUE}Installing the Magics project and dependencies...${NC}"
    pip install -e .
fi

# Display installed packages
echo -e "${GREEN}Installation complete! The following packages were installed:${NC}"
pip list | grep -E 'numpy|pyzmq|gymnasium|magics-gym'

echo -e "${GREEN}Virtual environment setup complete!${NC}"
echo -e "${GREEN}To activate the virtual environment in the future, run:${NC}"

# Show activation instructions based on shell
if [ "$SHELL_TYPE" = "fish" ]; then
    echo -e "  ${BLUE}source .venv/bin/activate.fish${NC} (for Fish shell)"
else
    echo -e "  ${BLUE}source .venv/bin/activate${NC} (on Unix/Linux/MacOS)"
    echo -e "  ${BLUE}source .venv/Scripts/activate${NC} (on Windows Git Bash)"
    echo -e "  ${BLUE}.venv\\Scripts\\activate${NC} (on Windows cmd)"
    echo -e "  ${BLUE}.venv\\Scripts\\Activate.ps1${NC} (on Windows PowerShell)"
fi