#!/usr/bin/env python3
"""
Generate human-readable reports from comparison results.
Usage: generate-report.py <comparison-json-file> [output-dir]
"""

import json
import sys
import os
from pathlib import Path
from typing import Dict, Any


def generate_markdown_report(comparison_data: Dict[str, Any], output_dir: Path) -> None:
    """Generate Markdown report from comparison data."""
    report_path = output_dir / 'summary.md'
    
    with open(report_path, 'w') as f:
        f.write("# Validation Tool Comparison Report\n\n")
        f.write(f"**Generated:** {comparison_data.get('timestamp', 'Unknown')}\n\n")
        
        # Summary section
        f.write("## Summary\n\n")
        summary = comparison_data.get('summary', {})
        f.write(f"- **Total Files Analyzed:** {summary.get('total_files', 0)}\n")
        f.write(f"- **Total Errors (Truss):** {summary.get('total_errors_truss', 0)}\n")
        f.write(f"- **Total Errors (actionlint):** {summary.get('total_errors_actionlint', 0)}\n")
        f.write(f"- **Truss Coverage:** {summary.get('coverage_truss', 0.0):.1%}\n")
        f.write(f"- **Avg Time (Truss):** {summary.get('avg_time_truss_ms', 0.0):.2f}ms\n")
        f.write(f"- **Avg Time (actionlint):** {summary.get('avg_time_actionlint_ms', 0.0):.2f}ms\n")
        f.write(f"- **Speedup:** {summary.get('avg_time_actionlint_ms', 1.0) / max(summary.get('avg_time_truss_ms', 0.1), 0.1):.1f}x\n\n")
        
        # Coverage Analysis
        f.write("## Coverage Analysis\n\n")
        coverage = comparison_data.get('coverage_analysis', {})
        for tool, data in coverage.items():
            f.write(f"### {tool}\n\n")
            f.write(f"- **Total Errors:** {data.get('total_errors', 0)}\n")
            f.write(f"- **Truss Found:** {data.get('truss_found', 0)}\n")
            f.write(f"- **Coverage:** {data.get('coverage', 0.0):.1%}\n\n")
        
        # Tool Statistics
        f.write("## Tool Statistics\n\n")
        tools = comparison_data.get('tools', {})
        f.write("| Tool | Files Analyzed | Errors Found | Avg Time (ms) | Total Time (ms) |\n")
        f.write("|------|----------------|--------------|---------------|-----------------|\n")
        
        for tool, stats in tools.items():
            f.write(f"| {tool} | {stats.get('files_analyzed', 0)} | "
                   f"{stats.get('errors_found', 0)} | "
                   f"{stats.get('avg_time_ms', 0.0):.2f} | "
                   f"{stats.get('total_time_ms', 0.0):.2f} |\n")
        f.write("\n")
        
        # File-by-File Breakdown
        f.write("## File-by-File Breakdown\n\n")
        files = comparison_data.get('files', [])
        
        for file_data in files[:20]:  # Limit to first 20 files
            f.write(f"### {file_data.get('path', 'Unknown')}\n\n")
            
            # Tool results
            f.write("**Tool Results:**\n")
            for tool, tool_data in file_data.get('tools', {}).items():
                f.write(f"- **{tool}:** {tool_data.get('errors', 0)} errors, "
                       f"{tool_data.get('duration_ms', 0.0):.2f}ms\n")
            f.write("\n")
            
            # Comparison
            comparison = file_data.get('comparison', {})
            if comparison:
                f.write("**Comparison:**\n")
                for comp_tool, comp_data in comparison.items():
                    f.write(f"- vs **{comp_tool}:** {comp_data.get('errors_in_common', 0)} common, "
                           f"{comp_data.get('unique_to_truss', 0)} unique to Truss, "
                           f"{comp_data.get('unique_to_competitor', 0)} unique to {comp_tool}\n")
            f.write("\n")
        
        if len(files) > 20:
            f.write(f"\n*... and {len(files) - 20} more files*\n")
    
    print(f"Markdown report written to: {report_path}", file=sys.stderr)


def generate_html_report(comparison_data: Dict[str, Any], output_dir: Path) -> None:
    """Generate HTML report from comparison data."""
    report_path = output_dir / 'summary.html'
    
    summary = comparison_data.get('summary', {})
    tools = comparison_data.get('tools', {})
    coverage = comparison_data.get('coverage_analysis', {})
    
    html = f"""<!DOCTYPE html>
<html>
<head>
    <title>Validation Tool Comparison Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        h1 {{ color: #333; }}
        h2 {{ color: #666; margin-top: 30px; }}
        table {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .metric {{ margin: 10px 0; }}
        .coverage-good {{ color: green; }}
        .coverage-medium {{ color: orange; }}
        .coverage-poor {{ color: red; }}
    </style>
</head>
<body>
    <h1>Validation Tool Comparison Report</h1>
    <p><strong>Generated:</strong> {comparison_data.get('timestamp', 'Unknown')}</p>
    
    <h2>Summary</h2>
    <div class="metric"><strong>Total Files Analyzed:</strong> {summary.get('total_files', 0)}</div>
    <div class="metric"><strong>Total Errors (Truss):</strong> {summary.get('total_errors_truss', 0)}</div>
    <div class="metric"><strong>Total Errors (actionlint):</strong> {summary.get('total_errors_actionlint', 0)}</div>
    <div class="metric"><strong>Truss Coverage:</strong> <span class="coverage-good">{summary.get('coverage_truss', 0.0):.1%}</span></div>
    <div class="metric"><strong>Avg Time (Truss):</strong> {summary.get('avg_time_truss_ms', 0.0):.2f}ms</div>
    <div class="metric"><strong>Avg Time (actionlint):</strong> {summary.get('avg_time_actionlint_ms', 0.0):.2f}ms</div>
    <div class="metric"><strong>Speedup:</strong> {summary.get('avg_time_actionlint_ms', 1.0) / max(summary.get('avg_time_truss_ms', 0.1), 0.1):.1f}x</div>
    
    <h2>Coverage Analysis</h2>
    <table>
        <tr>
            <th>Tool</th>
            <th>Total Errors</th>
            <th>Truss Found</th>
            <th>Coverage</th>
        </tr>
"""
    
    for tool, data in coverage.items():
        cov = data.get('coverage', 0.0)
        cov_class = 'coverage-good' if cov >= 0.9 else 'coverage-medium' if cov >= 0.7 else 'coverage-poor'
        html += f"""        <tr>
            <td>{tool}</td>
            <td>{data.get('total_errors', 0)}</td>
            <td>{data.get('truss_found', 0)}</td>
            <td class="{cov_class}">{cov:.1%}</td>
        </tr>
"""
    
    html += """    </table>
    
    <h2>Tool Statistics</h2>
    <table>
        <tr>
            <th>Tool</th>
            <th>Files Analyzed</th>
            <th>Errors Found</th>
            <th>Avg Time (ms)</th>
            <th>Total Time (ms)</th>
        </tr>
"""
    
    for tool, stats in tools.items():
        html += f"""        <tr>
            <td>{tool}</td>
            <td>{stats.get('files_analyzed', 0)}</td>
            <td>{stats.get('errors_found', 0)}</td>
            <td>{stats.get('avg_time_ms', 0.0):.2f}</td>
            <td>{stats.get('total_time_ms', 0.0):.2f}</td>
        </tr>
"""
    
    html += """    </table>
</body>
</html>
"""
    
    with open(report_path, 'w') as f:
        f.write(html)
    
    print(f"HTML report written to: {report_path}", file=sys.stderr)


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: generate-report.py <comparison-json-file> [output-dir]", file=sys.stderr)
        sys.exit(1)
    
    comparison_file = sys.argv[1]
    output_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else Path('test-suite/comparison/reports')
    
    if not os.path.exists(comparison_file):
        print(f"Error: Comparison file not found: {comparison_file}", file=sys.stderr)
        sys.exit(1)
    
    output_dir.mkdir(parents=True, exist_ok=True)
    
    with open(comparison_file, 'r') as f:
        comparison_data = json.load(f)
    
    generate_markdown_report(comparison_data, output_dir)
    generate_html_report(comparison_data, output_dir)

