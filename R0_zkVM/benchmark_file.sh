#!/bin/bash

# Directory containing notebooks
notebooks_dir="notebooks"

# The output JSON file
output_file="benchmarks.json"

# Initialize an empty JSON object
json_object="{}"

# Iterate over subdirectories in the notebooks directory
for subdir in "$notebooks_dir"/*; do
    if [[ -d "$subdir" ]]; then
        subdir_name=$(basename "$subdir")

        # For each subdirectory, create a JSON object for its notebooks
        subdir_object="{}"
        for notebook in "$subdir"/*.ipynb; do
            if [[ -f "$notebook" ]]; then
                notebook_name=$(basename "$notebook" .ipynb)

                # Add provingTime and verifyTime fields to the notebook object
                subdir_object=$(jq -n \
                    --arg name "$notebook_name" \
                    --argjson obj "$subdir_object" \
                    '$obj + {($name): {"provingTime": null, "verifyTime": null}}')
            fi
        done

        # Merge the subdirectory object with the main JSON object
        json_object=$(jq -n \
            --argjson obj "$json_object" \
            --arg subdir "$subdir_name" \
            --argjson subdir_obj "$subdir_object" \
            '$obj + {($subdir): $subdir_obj}')
    fi
done

# Save the JSON object to the output file
echo "$json_object" | jq '.' > "$output_file"
