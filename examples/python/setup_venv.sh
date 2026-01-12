#!/bin/bash
# Setup virtual environment for Wasmlette Python client

set -e

echo "Setting up Python virtual environment..."

# Create venv if it doesn't exist
if [ ! -d "venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv venv
else
    echo "Virtual environment already exists."
fi

# Activate and install dependencies
echo "Installing dependencies..."
./venv/bin/pip install --upgrade pip
./venv/bin/pip install -r requirements.txt

echo ""
echo "✓ Setup complete!"
echo ""
echo "To activate the virtual environment:"
echo "  source venv/bin/activate"
echo ""
echo "To run the example:"
echo "  ./venv/bin/python simple_example.py"
echo "  # or after activating: python simple_example.py"
echo ""
echo "To run tests:"
echo "  cd ../../tests/integration"
echo "  ../../examples/python/venv/bin/python -m pytest test_rpc_server.py -v"
