hyperfine --warmup 3 'cargo run --bin with_script -- --disable-window' 'cargo run --bin no_script -- --disable-window' --export-json benchmark_data.json

# Create a virtual environment in your project directory
python3 -m venv venv

# Activate it
source venv/bin/activate

# Now install matplotlib
pip install matplotlib

# Run your script
python3 plot_histogram.py benchmark_data.json

# When you're done, deactivate the environment
deactivate