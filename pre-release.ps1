# Pre-release checks script for Rust libraries
Write-Host "🔍 Starting pre-release checks..." -ForegroundColor Cyan

# Function to run a command and check its exit code
function Invoke-CommandWithCheck {
    param (
        [string]$Message,
        [scriptblock]$Command
    )
    Write-Host "`n► $Message" -ForegroundColor Yellow
    & $Command
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Failed: $Message" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    Write-Host "✅ Passed: $Message" -ForegroundColor Green
}

# Update dependencies
Invoke-CommandWithCheck "Updating dependencies" {
    cargo update
}

# Format check
Invoke-CommandWithCheck "Checking code formatting" {
    cargo fmt --all -- --check
}

# Clippy with all features
Invoke-CommandWithCheck "Running clippy checks" {
    cargo clippy --all-features -- -D warnings
}

# Run tests
Invoke-CommandWithCheck "Running tests" {
    cargo test --all-features
}

# Check documentation
Invoke-CommandWithCheck "Checking documentation" {
    cargo doc --no-deps --all-features
}

# Run cargo check with all features
Invoke-CommandWithCheck "Running cargo check" {
    cargo check --all-features
}

# Verify the package
Invoke-CommandWithCheck "Verifying package" {
    cargo package --no-verify --allow-dirty
}

# # Clean up any artifacts
# Invoke-CommandWithCheck "Cleaning up" {
#     cargo clean
# }

Write-Host "`n✨ Pre-release checks completed successfully! ✨" -ForegroundColor Cyan
Write-Host "You can now proceed with publishing your release.`n"

# Show next steps
Write-Host "Next steps:" -ForegroundColor Magenta
Write-Host "1. Update version in Cargo.toml"
Write-Host "2. Commit changes: git commit -am 'Release v[VERSION]'"
Write-Host "3. Create git tag: git tag v[VERSION]"
Write-Host "4. Push changes: git push && git push --tags"
Write-Host "5. Publish to crates.io: cargo publish" 