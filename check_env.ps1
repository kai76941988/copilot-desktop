# Pake Environment Check Script
# Target Path: E:\miniconda3

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Pake Environment Check" -ForegroundColor Cyan
Write-Host "  Target Path: E:\miniconda3" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check Node.js
Write-Host "=== 1. Node.js Check ===" -ForegroundColor Cyan
try {
    $nodeVersion = node --version 2>$null
    $nodePath = (Get-Command node -ErrorAction SilentlyContinue).Source
    
    if ($nodeVersion) {
        Write-Host "[OK] Node.js version: $nodeVersion" -ForegroundColor Green
        Write-Host "     Path: $nodePath" -ForegroundColor White
        
        if ($nodePath -like "E:\miniconda3*") {
            Write-Host "[PASS] Node.js path is correct" -ForegroundColor Green
        } else {
            Write-Host "[WARN] Node.js NOT in E:\miniconda3" -ForegroundColor Yellow
            Write-Host "       Current: $nodePath" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "[ERROR] Node.js check failed" -ForegroundColor Red
}

Write-Host ""

# Check Rust
Write-Host "=== 2. Rust Check ===" -ForegroundColor Cyan
try {
    $rustVersion = rustc --version 2>$null
    
    if ($rustVersion) {
        Write-Host "[OK] Rust version: $rustVersion" -ForegroundColor Green
        
        $rustupHome = $env:RUSTUP_HOME
        $cargoHome = $env:CARGO_HOME
        
        Write-Host "     RUSTUP_HOME: $rustupHome" -ForegroundColor White
        Write-Host "     CARGO_HOME: $cargoHome" -ForegroundColor White
        
        if ($rustupHome -like "E:\miniconda3*" -and $cargoHome -like "E:\miniconda3*") {
            Write-Host "[PASS] Rust path is correct" -ForegroundColor Green
        } else {
            Write-Host "[WARN] Rust NOT in E:\miniconda3" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "[ERROR] Rust check failed" -ForegroundColor Red
}

Write-Host ""

# Check npm
Write-Host "=== 3. npm Path Check ===" -ForegroundColor Cyan
try {
    $npmPrefix = npm config get prefix 2>$null
    
    if ($npmPrefix) {
        Write-Host "[OK] npm prefix: $npmPrefix" -ForegroundColor Green
        
        if ($npmPrefix -like "E:\miniconda3*") {
            Write-Host "[PASS] npm path is correct" -ForegroundColor Green
        } else {
            Write-Host "[WARN] npm NOT in E:\miniconda3" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "[ERROR] npm check failed" -ForegroundColor Red
}

Write-Host ""

# Check pnpm
Write-Host "=== 4. pnpm Check ===" -ForegroundColor Cyan
try {
    $pnpmVersion = pnpm --version 2>$null
    
    if ($pnpmVersion) {
        Write-Host "[OK] pnpm version: $pnpmVersion" -ForegroundColor Green
        
        $pnpmStore = pnpm config get store-dir 2>$null
        Write-Host "     pnpm store: $pnpmStore" -ForegroundColor White
        
        if ($pnpmStore -like "E:\miniconda3*") {
            Write-Host "[PASS] pnpm path is correct" -ForegroundColor Green
        } else {
            Write-Host "[WARN] pnpm NOT in E:\miniconda3" -ForegroundColor Yellow
        }
    }
} catch {
    Write-Host "[WARN] pnpm not installed" -ForegroundColor Yellow
}

Write-Host ""

# Check Build Tools
Write-Host "=== 5. Build Tools Check ===" -ForegroundColor Cyan
try {
    $vsWherePath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    
    if (Test-Path $vsWherePath) {
        $vsInstallPath = & $vsWherePath -latest -property installationPath 2>$null
        if ($vsInstallPath) {
            Write-Host "[OK] Visual Studio: $vsInstallPath" -ForegroundColor Green
        }
    } else {
        Write-Host "[WARN] Visual Studio not found" -ForegroundColor Yellow
    }
    
    $sdkPath = "${env:ProgramFiles(x86)}\Windows Kits\10"
    if (Test-Path $sdkPath) {
        Write-Host "[OK] Windows 10 SDK found" -ForegroundColor Green
    } else {
        Write-Host "[WARN] Windows 10 SDK not found" -ForegroundColor Yellow
    }
    
} catch {
    Write-Host "[ERROR] Build tools check failed" -ForegroundColor Red
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Environment Check Complete" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
