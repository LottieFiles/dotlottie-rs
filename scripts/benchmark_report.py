import json
import os
from pathlib import Path

def analyze_criterion_output(criterion_dir):
    results = []
    
    for estimates_file in criterion_dir.rglob("*/change/estimates.json"):
        bench_name = estimates_file.parent.parent.name
        
        try:
            with open(estimates_file) as f:
                data = json.load(f)
            
            mean_change = data['mean']['point_estimate']
            mean_lower = data['mean']['confidence_interval']['lower_bound']
            mean_upper = data['mean']['confidence_interval']['upper_bound']
            
            results.append({
                'name': bench_name,
                'change': mean_change,
                'lower': mean_lower,
                'upper': mean_upper
            })
        except (FileNotFoundError, json.JSONDecodeError, KeyError):
            continue
    
    return results

# Analyze both packages
rs_results = analyze_criterion_output(Path("dotlottie-rs/target/criterion"))

all_results = []
if rs_results:
    all_results.extend([{**r, 'package': 'dotlottie-rs'} for r in rs_results])

# Generate markdown report
comment = "## 📊 Benchmark Results\n\n"

if not all_results:
    comment += "No benchmark comparisons found. Benchmarks may not be set up yet.\n"
    has_regression = False
else:
    # Group by package
    by_package = {}
    for r in all_results:
        pkg = r['package']
        if pkg not in by_package:
            by_package[pkg] = []
        by_package[pkg].append(r)
    
    has_regression = False
    
    for pkg, results in sorted(by_package.items()):
        comment += f"### {pkg}\n\n"
        comment += "| Benchmark | Change | Confidence Interval | Status |\n"
        comment += "|-----------|--------|---------------------|--------|\n"
        
        for result in sorted(results, key=lambda x: x['change'], reverse=True):
            change_pct = result['change'] * 100
            lower_pct = result['lower'] * 100
            upper_pct = result['upper'] * 100
            
            # Determine status
            if result['lower'] > 0.05:  # Significantly slower
                status = "⚠️ Regression"
                has_regression = True
            elif result['upper'] < -0.05:  # Significantly faster
                status = "🚀 Improvement"
            elif abs(result['change']) < 0.02:  # Within 2%
                status = "✅ No change"
            else:
                status = "⚡ Slight change"
            
            comment += f"| `{result['name']}` | **{change_pct:+.2f}%** | [{lower_pct:+.2f}%, {upper_pct:+.2f}%] | {status} |\n"
        
        comment += "\n"
    
    if has_regression:
        comment += "⚠️ **Warning:** Performance regressions detected (>5% slower with 95% confidence)!\n\n"
    
    comment += "<details>\n<summary>How to interpret these results</summary>\n\n"
    comment += "- **Change**: Estimated performance difference (negative = faster, positive = slower)\n"
    comment += "- **Confidence Interval**: 95% confidence bounds for the change\n"
    comment += "- **Regression**: Lower bound > +5% (confidently slower)\n"
    comment += "- **Improvement**: Upper bound < -5% (confidently faster)\n\n"
    comment += "Criterion uses statistical analysis to account for noise and provide reliable comparisons.\n"
    comment += "</details>\n"

# Write outputs
with open(os.environ['GITHUB_OUTPUT'], 'a') as f:
    f.write(f"comment<<EOF\n{comment}\nEOF\n")
    f.write(f"has_regression={'true' if has_regression else 'false'}\n")