#!/bin/bash
# Simple link checker for markdown files

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[✓]${NC} $1"
}

print_error() {
    echo -e "${RED}[✗]${NC} $1"
}

# Decode URL encoding
decode_url() {
    echo "$1" | sed 's/%20/ /g'
}

# Check if file exists
check_link() {
    local url="$1"
    local file="$2"
    
    # Skip external URLs, mailto, anchors
    [[ "$url" =~ ^(https?://|mailto:|#) ]] && return 0
    
    # Decode URL
    url=$(decode_url "$url")
    
    # Check absolute paths
    if [[ "$url" =~ ^/ ]]; then
        if [[ -e "$url" ]]; then
            return 0
        else
            print_error "Missing: $url (in $file)"
            return 1
        fi
    else
        # Check relative paths
        local dir=$(dirname "$file")
        local full_path="$dir/$url"
        if [[ -e "$full_path" ]]; then
            return 0
        else
            print_error "Missing: $url -> $full_path (in $file)"
            return 1
        fi
    fi
}

# Main
total_errors=0
files_checked=0
links_checked=0

print_status "🔗 Checking markdown links..."

for file in ./README.md ./docs/DEVELOPMENT.md ./PROJECT_SUMMARY.md; do
    if [[ -f "$file" ]]; then
        ((files_checked++))
        print_status "Checking $file"
        
        # Find links and check them
        while IFS= read -r line; do
            if [[ -n "$line" ]]; then
                url=$(echo "$line" | sed 's/.*](\([^)]*\)).*/\1/')
                ((links_checked++))
                if ! check_link "$url" "$file"; then
                    ((total_errors++))
                fi
            fi
        done < <(grep -o '\[[^]]*\]([^)]*)' "$file" 2>/dev/null)
    fi
done

print_status "Checked $links_checked links in $files_checked files"

if [[ $total_errors -eq 0 ]]; then
    print_success "All links are valid"
    exit 0
else
    print_error "Found $total_errors broken links"
    exit 1
fi