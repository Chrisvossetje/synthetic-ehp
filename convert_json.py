#!/usr/bin/env python3
import json

# Read the old page.json file
with open('/Users/chris/Documents/GitHub/comodules-web/site/static/page.json', 'r') as f:
    old_data = json.load(f)

# Convert to new SSeq format
new_sseq = {
    "name": old_data["name"],
    "degrees": old_data["degrees"],
    "x_formula": old_data["x_formula"],
    "y_formula": old_data["y_formula"],
    "pages": [
        {
            "id": 2,  # Set page id to 2 as requested
            "generators": old_data["generators"],
            "structure_lines": old_data["structure_lines"]
        }
    ],
    "differentials": old_data["differentials"]
}

# Write the new sseq.json file
with open('/Users/chris/Documents/GitHub/comodules-web/site/static/sseq.json', 'w') as f:
    json.dump(new_sseq, f, indent=2)

print("Successfully converted page.json to sseq.json with proper SSeq structure")