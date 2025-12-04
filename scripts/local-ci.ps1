# Local CI Script
# Runs the full CI pipeline locally, including Frontend and Backend checks.

$ErrorActionPreference = 'Stop'

function Write-Header {
    param([string]$Message)
    Write-Host "`n======================================================================" -ForegroundColor Cyan
    Write-Host " $Message" -ForegroundColor Cyan
    Write-Host "======================================================================`n" -ForegroundColor Cyan
}

function Write-Success {
    param([string]$Message)
    Write-Host "✅ $Message" -ForegroundColor Green
}

function Write-ErrorMsg {
    param([string]$Message)
    Write-Host "❌ $Message" -ForegroundColor Red
}

function Check-Command {
    param([string]$Name, [string]$Command)
    try {
        Invoke-Expression "$Command --version" | Out-Null
        Write-Success "Found $Name"
    } catch {
        Write-ErrorMsg "Missing $Name. Please install it and try again."
        exit 1
    }
}

# 1. Environment Check
Write-Header "Checking Environment"
Check-Command "Node.js" "node"
Check-Command "pnpm" "pnpm"
Check-Command "Rust (cargo)" "cargo"

# 2. Install Dependencies
Write-Header "Installing Dependencies"
try {
    pnpm install
    if ($LASTEXITCODE -ne 0) { throw "pnpm install failed" }
    Write-Success "Dependencies installed"
} catch {
    Write-ErrorMsg "Failed to install dependencies: $_"
    exit 1
}

# 3. Run CI Pipeline
Write-Header "Running CI Pipeline (Lint, Typecheck, Test, Build)"
Write-Host "Running: pnpm turbo run lint lint:rust typecheck check:rust test test:rust build" -ForegroundColor Gray

try {
    # Run all checks in parallel where possible via Turbo
    # We explicitly list all tasks to ensure everything is covered
    pnpm turbo run lint lint:rust typecheck check:rust test test:rust build
    
    if ($LASTEXITCODE -ne 0) { throw "Turbo pipeline failed" }
    
    Write-Header "🎉 CI Passed Successfully!"
    Write-Success "All checks passed. You are ready to push!"
    
    # Future hooks
    # Write-Host "Ready for deployment..." -ForegroundColor Gray
} catch {
    Write-Header "💥 CI Failed"
    Write-ErrorMsg "The pipeline encountered errors. Please fix them before pushing."
    exit 1
}
