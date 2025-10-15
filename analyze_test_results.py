#!/usr/bin/env python3
"""XM125 Test Results Analysis - Statistical validation and performance metrics"""

import sys
import csv
import json
import glob
import statistics
from pathlib import Path
from datetime import datetime

def analyze_stability_data(csv_file):
    """Analyze continuous measurement stability"""
    try:
        with open(csv_file, 'r') as f:
            reader = csv.DictReader(f)
            measurements = list(reader)
        
        if not measurements:
            return {"error": "No measurement data found"}
        
        # Extract numeric values
        distances = [float(m.get('distance', 0)) for m in measurements if m.get('distance')]
        presence_scores = [float(m.get('intra_score', 0)) for m in measurements if m.get('intra_score')]
        
        return {
            "total_measurements": len(measurements),
            "distance_stats": {
                "mean": statistics.mean(distances) if distances else 0,
                "stdev": statistics.stdev(distances) if len(distances) > 1 else 0,
                "min": min(distances) if distances else 0,
                "max": max(distances) if distances else 0
            },
            "presence_stats": {
                "mean": statistics.mean(presence_scores) if presence_scores else 0,
                "stdev": statistics.stdev(presence_scores) if len(presence_scores) > 1 else 0,
                "detection_rate": sum(1 for m in measurements if m.get('presence') == '1') / len(measurements) * 100
            }
        }
    except Exception as e:
        return {"error": str(e)}

def analyze_log_files(log_dir):
    """Analyze test log files for pass/fail status"""
    results = {}
    log_files = glob.glob(f"{log_dir}/*.log")
    
    for log_file in log_files:
        test_name = Path(log_file).stem
        try:
            with open(log_file, 'r') as f:
                content = f.read()
            
            # Simple pass/fail detection based on common error patterns
            errors = [
                "ERROR", "FAILED", "error:", "failed:", 
                "timeout", "not found", "permission denied"
            ]
            
            has_errors = any(error.lower() in content.lower() for error in errors)
            
            results[test_name] = {
                "status": "FAIL" if has_errors else "PASS",
                "log_size": len(content),
                "error_indicators": [e for e in errors if e.lower() in content.lower()]
            }
        except Exception as e:
            results[test_name] = {"status": "ERROR", "error": str(e)}
    
    return results

def generate_report(test_dir):
    """Generate comprehensive test report"""
    report = {
        "timestamp": datetime.now().isoformat(),
        "test_directory": test_dir,
        "summary": {"total_tests": 0, "passed": 0, "failed": 0, "errors": 0}
    }
    
    # Analyze log files
    log_analysis = analyze_log_files(test_dir)
    report["test_results"] = log_analysis
    
    # Count results
    for test, result in log_analysis.items():
        report["summary"]["total_tests"] += 1
        if result["status"] == "PASS":
            report["summary"]["passed"] += 1
        elif result["status"] == "FAIL":
            report["summary"]["failed"] += 1
        else:
            report["summary"]["errors"] += 1
    
    # Analyze stability data if available
    stability_file = Path(test_dir) / "stability_test.csv"
    if stability_file.exists():
        report["stability_analysis"] = analyze_stability_data(stability_file)
    
    # Calculate pass rate
    total = report["summary"]["total_tests"]
    passed = report["summary"]["passed"]
    report["summary"]["pass_rate"] = (passed / total * 100) if total > 0 else 0
    
    return report

def main():
    if len(sys.argv) != 2:
        print("Usage: python3 analyze_test_results.py <test_results_directory>")
        sys.exit(1)
    
    test_dir = sys.argv[1]
    if not Path(test_dir).exists():
        print(f"Error: Test directory '{test_dir}' not found")
        sys.exit(1)
    
    print(f"Analyzing test results from: {test_dir}")
    
    # Generate report
    report = generate_report(test_dir)
    
    # Save detailed report
    report_file = Path(test_dir) / "analysis_report.json"
    with open(report_file, 'w') as f:
        json.dump(report, f, indent=2)
    
    # Print summary
    summary = report["summary"]
    print(f"\n📊 Test Results Summary")
    print(f"{'='*50}")
    print(f"Total Tests: {summary['total_tests']}")
    print(f"Passed: {summary['passed']} ✅")
    print(f"Failed: {summary['failed']} ❌")
    print(f"Errors: {summary['errors']} ⚠️")
    print(f"Pass Rate: {summary['pass_rate']:.1f}%")
    
    # Stability analysis summary
    if "stability_analysis" in report and "error" not in report["stability_analysis"]:
        stability = report["stability_analysis"]
        print(f"\n📈 Stability Analysis")
        print(f"{'='*50}")
        print(f"Measurements: {stability['total_measurements']}")
        if stability['distance_stats']['stdev'] > 0:
            print(f"Distance StdDev: {stability['distance_stats']['stdev']:.3f}m")
        if 'detection_rate' in stability['presence_stats']:
            print(f"Detection Rate: {stability['presence_stats']['detection_rate']:.1f}%")
    
    print(f"\nDetailed report saved to: {report_file}")
    
    # Exit with appropriate code
    if summary['failed'] > 0 or summary['errors'] > 0:
        print("\n⚠️  Some tests failed - review detailed report")
        sys.exit(1)
    else:
        print("\n🎉 All tests passed successfully!")
        sys.exit(0)

if __name__ == "__main__":
    main()