VISUALIZATION SETUP INSTRUCTIONS
=================================

The visualization script is ready but requires Python packages.

OPTION 1: Install packages and run
-----------------------------------
python3 -m venv venv
source venv/bin/activate
pip install pandas matplotlib numpy
python3 generate_visualizations.py

OPTION 2: Use the CSV files with other tools
---------------------------------------------
- Import CSVs into Excel/Google Sheets
- Use R for plotting
- Use online tools (plotly, datawrapper)

CSV FILES AVAILABLE:
- pareto_frontier.csv       (speed vs size)
- heatmap_data.csv          (benchmark x profile matrix)
- profile_rankings.csv      (top profiles with CIs)
- workload_comparison.csv   (workload analysis)

OPTION 3: View data in terminal
--------------------------------
# Simple text visualization
python3 analyze_speedups.py

# View CSV contents
column -t -s, pareto_frontier.csv | head -10

See VISUALIZATION_GUIDE.md for complete instructions.
