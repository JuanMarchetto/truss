#!/usr/bin/env python3
"""
Compare validation results across multiple tools.
Usage: compare-results.py <results-dir> [output-json-file]
"""

import json
import sys
import os
from pathlib import Path
from collections import defaultdict
from typing import Dict, List, Set, Tuple, Any
from difflib import SequenceMatcher
from datetime import datetime


def normalize_error_location(diagnostic: Dict[str, Any], file_path: str) -> Tuple[str, int, int]:
    """Normalize error location to (file, line, column) tuple."""
    location = diagnostic.get('location', {})
    if isinstance(location, dict):
        line = location.get('line', 0)
        column = location.get('column', 0)
    else:
        # Handle span-based locations (Truss format)
        span = diagnostic.get('span', {})
        if isinstance(span, dict):
            # Approximate line/column from span (simplified)
            line = 1
            column = span.get('start', 0)
        else:
            line = 1
            column = 0
    
    return (file_path, line, column)


def normalize_error_message(message: str) -> str:
    """Normalize error message for comparison."""
    # Lowercase and remove extra whitespace
    normalized = ' '.join(message.lower().split())
    # Remove file paths (they vary)
    import re
    normalized = re.sub(r'[a-zA-Z0-9_\-/]+\.ya?ml', '<file>', normalized)
    return normalized


def similarity_score(msg1: str, msg2: str) -> float:
    """Calculate similarity between two error messages (0.0 to 1.0)."""
    return SequenceMatcher(None, msg1, msg2).ratio()


def match_errors(
    errors1: List[Dict[str, Any]],
    errors2: List[Dict[str, Any]],
    file_path: str,
    similarity_threshold: float = 0.7
) -> Tuple[List[Dict], List[Dict], List[Dict]]:
    """
    Match errors between two tools.
    Returns: (common_errors, unique_to_1, unique_to_2)
    """
    matched_indices_2 = set()
    common = []
    unique_1 = []
    
    for err1 in errors1:
        loc1 = normalize_error_location(err1, file_path)
        msg1 = normalize_error_message(err1.get('message', ''))
        
        best_match = None
        best_score = 0.0
        best_idx = -1
        
        for idx, err2 in enumerate(errors2):
            if idx in matched_indices_2:
                continue
            
            loc2 = normalize_error_location(err2, file_path)
            msg2 = normalize_error_message(err2.get('message', ''))
            
            # Check location match (same file and line)
            location_match = loc1[0] == loc2[0] and abs(loc1[1] - loc2[1]) <= 2
            
            # Check message similarity
            msg_score = similarity_score(msg1, msg2)
            
            # Combined score
            if location_match and msg_score > best_score:
                best_score = msg_score
                best_match = err2
                best_idx = idx
        
        if best_match and best_score >= similarity_threshold:
            common.append({
                'tool1': err1,
                'tool2': best_match,
                'similarity': best_score
            })
            matched_indices_2.add(best_idx)
        else:
            unique_1.append(err1)
    
    # Errors unique to tool 2
    unique_2 = [
        err2 for idx, err2 in enumerate(errors2)
        if idx not in matched_indices_2
    ]
    
    return common, unique_1, unique_2


def load_tool_results(results_dir: Path, tool_name: str) -> Dict[str, Any]:
    """Load results for a specific tool."""
    tool_dir = results_dir / tool_name
    if not tool_dir.exists():
        return {}
    
    results = {}
    for result_file in tool_dir.glob('*.json'):
        try:
            with open(result_file, 'r') as f:
                content = f.read().strip()
                if not content:
                    continue
                data = json.loads(content)
                # Handle both single file results and arrays
                if isinstance(data, list):
                    for item in data:
                        file_path = item.get('file', '')
                        if file_path and 'error' not in item:
                            results[file_path] = item
                else:
                    file_path = data.get('file', '')
                    # Skip results with errors (tool not found, etc.)
                    if file_path and 'error' not in data:
                        results[file_path] = data
        except json.JSONDecodeError as e:
            print(f"Warning: Invalid JSON in {result_file}: {e}", file=sys.stderr)
            # Try to see what's in the file
            try:
                with open(result_file, 'r') as f:
                    preview = f.read(200)
                    print(f"  Preview: {preview}", file=sys.stderr)
            except:
                pass
        except Exception as e:
            print(f"Warning: Failed to load {result_file}: {e}", file=sys.stderr)
    
    return results


def compare_results(results_dir: str, output_file: str = None) -> Dict[str, Any]:
    """Compare validation results across all tools."""
    results_path = Path(results_dir)
    if not results_path.exists():
        raise ValueError(f"Results directory not found: {results_dir}")
    
    # Load results from all tools
    tools = ['truss', 'actionlint', 'yamllint', 'yaml-language-server']
    tool_results = {}
    
    for tool in tools:
        results = load_tool_results(results_path, tool)
        tool_results[tool] = results
        result_count = len(results)
        if result_count > 0:
            print(f"Loaded {result_count} results for {tool}", file=sys.stderr)
        else:
            print(f"Warning: No valid results found for {tool}", file=sys.stderr)
    
    # Get all files that were analyzed
    all_files = set()
    for tool_data in tool_results.values():
        all_files.update(tool_data.keys())
    
    # Compare results file by file
    file_comparisons = []
    tool_stats = {tool: {
        'errors_found': 0,
        'files_analyzed': 0,
        'total_time_ms': 0.0,
        'avg_time_ms': 0.0
    } for tool in tools}
    
    for file_path in sorted(all_files):
        file_comparison = {
            'path': file_path,
            'tools': {},
            'comparison': {}
        }
        
        # Collect errors from each tool
        tool_errors = {}
        for tool in tools:
            if file_path in tool_results[tool]:
                result = tool_results[tool][file_path]
                errors = result.get('diagnostics', [])
                # Filter to only errors (not warnings/info)
                errors = [
                    e for e in errors
                    if e.get('severity', 'error').lower() == 'error'
                ]
                tool_errors[tool] = errors
                
                # Update stats
                tool_stats[tool]['errors_found'] += len(errors)
                tool_stats[tool]['files_analyzed'] += 1
                tool_stats[tool]['total_time_ms'] += result.get('duration_ms', 0.0)
                
                file_comparison['tools'][tool] = {
                    'errors': len(errors),
                    'duration_ms': result.get('duration_ms', 0.0),
                    'valid': result.get('valid', True)
                }
            else:
                tool_errors[tool] = []
                file_comparison['tools'][tool] = {
                    'errors': 0,
                    'duration_ms': 0.0,
                    'valid': True
                }
        
        # Compare Truss against each competitor
        truss_errors = tool_errors.get('truss', [])
        comparison_data = {}
        
        for competitor in ['actionlint', 'yamllint', 'yaml-language-server']:
            if competitor not in tool_errors:
                continue
            
            comp_errors = tool_errors[competitor]
            common, unique_truss, unique_comp = match_errors(
                truss_errors, comp_errors, file_path
            )
            
            comparison_data[competitor] = {
                'errors_in_common': len(common),
                'unique_to_truss': len(unique_truss),
                'unique_to_competitor': len(unique_comp),
                'truss_coverage': len(common) / len(comp_errors) if comp_errors else 1.0
            }
        
        file_comparison['comparison'] = comparison_data
        file_comparisons.append(file_comparison)
    
    # Calculate summary statistics
    for tool in tools:
        stats = tool_stats[tool]
        if stats['files_analyzed'] > 0:
            stats['avg_time_ms'] = stats['total_time_ms'] / stats['files_analyzed']
    
    # Overall coverage analysis
    coverage_analysis = {}
    for competitor in ['actionlint', 'yamllint', 'yaml-language-server']:
        if competitor not in tool_results:
            continue
        
        total_comp_errors = tool_stats[competitor]['errors_found']
        total_common = sum(
            fc['comparison'].get(competitor, {}).get('errors_in_common', 0)
            for fc in file_comparisons
        )
        
        coverage_analysis[competitor] = {
            'total_errors': total_comp_errors,
            'truss_found': total_common,
            'coverage': total_common / total_comp_errors if total_comp_errors > 0 else 1.0
        }
    
    # Build final comparison result
    result = {
        'timestamp': datetime.utcnow().isoformat() + 'Z',
        'files_analyzed': len(all_files),
        'tools': tool_stats,
        'coverage_analysis': coverage_analysis,
        'files': file_comparisons,
        'summary': {
            'total_files': len(all_files),
            'total_errors_truss': tool_stats['truss']['errors_found'],
            'total_errors_actionlint': tool_stats.get('actionlint', {}).get('errors_found', 0),
            'coverage_truss': coverage_analysis.get('actionlint', {}).get('coverage', 0.0),
            'avg_time_truss_ms': tool_stats['truss']['avg_time_ms'],
            'avg_time_actionlint_ms': tool_stats.get('actionlint', {}).get('avg_time_ms', 0.0)
        }
    }
    
    # Output result
    output_json = json.dumps(result, indent=2)
    
    if output_file:
        with open(output_file, 'w') as f:
            f.write(output_json)
        print(f"Comparison results written to: {output_file}", file=sys.stderr)
    else:
        print(output_json)
    
    return result


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: compare-results.py <results-dir> [output-json-file]", file=sys.stderr)
        sys.exit(1)
    
    results_dir = sys.argv[1]
    output_file = sys.argv[2] if len(sys.argv) > 2 else None
    
    try:
        compare_results(results_dir, output_file)
    except Exception as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)

