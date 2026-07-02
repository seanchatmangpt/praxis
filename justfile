set shell := ["bash", "-uc"]

# Run only tests affected by changes
test-changed:
    timeout 30s cargo cicd test changed

# Check target directory size and prune
clean-stale:
    timeout 30s cargo cicd target prune

# Build the workspace
build:
    timeout 120s cargo build
