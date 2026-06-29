# Run only tests affected by changes
test-changed:
    cargo cicd test changed

# Check target directory size and prune
clean-stale:
    cargo cicd target prune

# Build the workspace
build:
    cargo build

