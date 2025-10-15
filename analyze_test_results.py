#!/usr/bin/env python3
"""
XM125 Test Data Analysis Tool
Author: Dynamic Devices Ltd
Version: 1.0

Analyzes CSV data from XM125 hardware tests to generate performance metrics
and visualization plots for range, sensitivity, and coverage analysis.
"""

import csv
import sys
import os
import argparse
import statistics
from datetime import datetime
from pathlib import Path

try:
    import matplotlib.pyplot as plt
    import numpy as np
    PLOTTING_AVAILABLE = True
except ImportError:
    PLOTTING_AVAILABLE = False
    print("Warning: matplotlib not available. Plotting disabled.")

class XM125TestAnalyzer:
    def __init__(self, test_dir):
        self.test_dir = Path(test_dir)
        self.results = {}
        
    def parse_csv_file(self, csv_file):
        """Parse XM125 monitor CSV output"""
        data = []
        try:
            with open(csv_file, 'r') as f:
                # Skip header if present
                content = f.read().strip()
                if not content:
                    return data
                    
                lines = content.split('\n')
                for line in lines:
                    if 'Presence:' in line:
                        # Parse human-readable format
                        parts = line.split('|')
                        if len(parts) >= 4:
                            presence = 'DETECTED' in parts[0]
                            distance = float(parts[1].split(':')[1].replace('m', '').strip())
                            intra = float(parts[2].split(':')[1].strip())
                            inter = float(parts[3].split(':')[1].strip())
                            data.append({
                                'presence': presence,
                                'distance': distance,
                                'intra': intra,
                                'inter': inter,
                                'timestamp': datetime.now()
                            })
        except Exception as e:
            print(f"Error parsing {csv_file}: {e}")
        
        return data
    
    def analyze_range_test(self):
        """Analyze range test results"""
        print("\n=== RANGE TEST ANALYSIS ===")
        
        range_files = list(self.test_dir.glob("range_*.csv"))
        if not range_files:
            print("No range test files found")
            return
        
        range_results = {}
        
        for file in range_files:
            # Extract distance from filename (e.g., range_2.0m.csv)
            distance_str = file.stem.split('_')[1].replace('m', '')
            try:
                distance = float(distance_str)
            except ValueError:
                continue
                
            data = self.parse_csv_file(file)
            if data:
                detections = sum(1 for d in data if d['presence'])
                total = len(data)
                success_rate = (detections / total) * 100 if total > 0 else 0
                
                # Calculate distance accuracy
                detected_distances = [d['distance'] for d in data if d['presence']]
                avg_measured_distance = statistics.mean(detected_distances) if detected_distances else 0
                distance_error = abs(avg_measured_distance - distance) if detected_distances else float('inf')
                
                range_results[distance] = {
                    'success_rate': success_rate,
                    'detections': detections,
                    'total': total,
                    'avg_distance': avg_measured_distance,
                    'distance_error': distance_error
                }
                
                print(f"{distance}m: {success_rate:.1f}% success ({detections}/{total}), "
                      f"Measured: {avg_measured_distance:.2f}m, Error: {distance_error:.2f}m")
        
        # Generate range performance plot
        if PLOTTING_AVAILABLE and range_results:
            self.plot_range_performance(range_results)
        
        return range_results
    
    def analyze_false_positives(self):
        """Analyze false positive test results"""
        print("\n=== FALSE POSITIVE ANALYSIS ===")
        
        fp_file = self.test_dir / "false_positive_test.csv"
        if not fp_file.exists():
            print("No false positive test file found")
            return
        
        data = self.parse_csv_file(fp_file)
        if not data:
            print("No data in false positive test file")
            return
        
        false_positives = sum(1 for d in data if d['presence'])
        total = len(data)
        fp_rate = (false_positives / total) * 100 if total > 0 else 0
        
        print(f"False Positive Rate: {fp_rate:.1f}% ({false_positives}/{total})")
        
        if fp_rate <= 5:
            print("✅ PASS: False positive rate acceptable")
        elif fp_rate <= 10:
            print("⚠️  WARN: False positive rate marginal")
        else:
            print("❌ FAIL: False positive rate too high")
        
        return fp_rate
    
    def analyze_motion_sensitivity(self):
        """Analyze motion sensitivity test results"""
        print("\n=== MOTION SENSITIVITY ANALYSIS ===")
        
        motion_types = ['slow', 'normal', 'fast']
        motion_results = {}
        
        for motion in motion_types:
            motion_file = self.test_dir / f"motion_{motion}.csv"
            if motion_file.exists():
                data = self.parse_csv_file(motion_file)
                if data:
                    detections = sum(1 for d in data if d['presence'])
                    total = len(data)
                    success_rate = (detections / total) * 100 if total > 0 else 0
                    
                    # Analyze motion scores
                    intra_scores = [d['intra'] for d in data if d['presence']]
                    inter_scores = [d['inter'] for d in data if d['presence']]
                    
                    avg_intra = statistics.mean(intra_scores) if intra_scores else 0
                    avg_inter = statistics.mean(inter_scores) if inter_scores else 0
                    
                    motion_results[motion] = {
                        'success_rate': success_rate,
                        'avg_intra': avg_intra,
                        'avg_inter': avg_inter
                    }
                    
                    print(f"{motion.capitalize()} motion: {success_rate:.1f}% success, "
                          f"Intra: {avg_intra:.2f}, Inter: {avg_inter:.2f}")
        
        return motion_results
    
    def plot_range_performance(self, range_results):
        """Generate range performance plots"""
        if not PLOTTING_AVAILABLE:
            return
        
        distances = sorted(range_results.keys())
        success_rates = [range_results[d]['success_rate'] for d in distances]
        distance_errors = [range_results[d]['distance_error'] for d in distances]
        
        fig, (ax1, ax2) = plt.subplots(2, 1, figsize=(10, 8))
        
        # Success rate plot
        ax1.plot(distances, success_rates, 'bo-', linewidth=2, markersize=8)
        ax1.axhline(y=90, color='g', linestyle='--', alpha=0.7, label='Target (90%)')
        ax1.axhline(y=70, color='y', linestyle='--', alpha=0.7, label='Minimum (70%)')
        ax1.set_xlabel('Distance (m)')
        ax1.set_ylabel('Detection Success Rate (%)')
        ax1.set_title('XM125 Detection Success Rate vs Distance')
        ax1.grid(True, alpha=0.3)
        ax1.legend()
        ax1.set_ylim(0, 100)
        
        # Distance accuracy plot
        ax2.plot(distances, distance_errors, 'ro-', linewidth=2, markersize=8)
        ax2.axhline(y=0.2, color='g', linestyle='--', alpha=0.7, label='Target (±20cm)')
        ax2.set_xlabel('Distance (m)')
        ax2.set_ylabel('Distance Error (m)')
        ax2.set_title('XM125 Distance Measurement Accuracy')
        ax2.grid(True, alpha=0.3)
        ax2.legend()
        
        plt.tight_layout()
        plt.savefig(self.test_dir / 'range_performance.png', dpi=300, bbox_inches='tight')
        plt.close()
        
        print(f"Range performance plot saved: {self.test_dir / 'range_performance.png'}")
    
    def generate_summary_report(self):
        """Generate comprehensive test summary"""
        print("\n" + "="*60)
        print("XM125 HARDWARE TEST ANALYSIS SUMMARY")
        print("="*60)
        print(f"Test Directory: {self.test_dir}")
        print(f"Analysis Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
        
        # Analyze all test types
        range_results = self.analyze_range_test()
        fp_rate = self.analyze_false_positives()
        motion_results = self.analyze_motion_sensitivity()
        
        # Overall assessment
        print("\n=== OVERALL ASSESSMENT ===")
        
        # Range assessment
        if range_results:
            max_range = max(d for d, r in range_results.items() if r['success_rate'] >= 70)
            print(f"Maximum effective range: {max_range}m")
            
            accuracy_ok = all(r['distance_error'] < 0.5 for r in range_results.values() 
                            if r['distance_error'] != float('inf'))
            print(f"Distance accuracy: {'✅ PASS' if accuracy_ok else '❌ FAIL'}")
        
        # False positive assessment
        if fp_rate is not None:
            fp_ok = fp_rate <= 10
            print(f"False positive rate: {'✅ PASS' if fp_ok else '❌ FAIL'} ({fp_rate:.1f}%)")
        
        # Motion sensitivity assessment
        if motion_results:
            motion_ok = all(r['success_rate'] >= 70 for r in motion_results.values())
            print(f"Motion sensitivity: {'✅ PASS' if motion_ok else '❌ FAIL'}")
        
        # Save summary to file
        summary_file = self.test_dir / 'analysis_summary.txt'
        with open(summary_file, 'w') as f:
            f.write(f"XM125 Test Analysis Summary\n")
            f.write(f"Generated: {datetime.now()}\n\n")
            f.write(f"Range Results: {range_results}\n")
            f.write(f"False Positive Rate: {fp_rate}%\n")
            f.write(f"Motion Results: {motion_results}\n")
        
        print(f"\nDetailed analysis saved to: {summary_file}")

def main():
    parser = argparse.ArgumentParser(description='Analyze XM125 hardware test results')
    parser.add_argument('test_dir', help='Directory containing test result files')
    parser.add_argument('--plot', action='store_true', help='Generate plots (requires matplotlib)')
    
    args = parser.parse_args()
    
    if not os.path.exists(args.test_dir):
        print(f"Error: Test directory '{args.test_dir}' not found")
        sys.exit(1)
    
    analyzer = XM125TestAnalyzer(args.test_dir)
    analyzer.generate_summary_report()

if __name__ == '__main__':
    main()
