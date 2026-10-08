$ErrorActionPreference = 'Stop'
$root = (Resolve-Path (Join-Path $PSScriptRoot '../..')).Path
$manifest = Join-Path $PSScriptRoot 'Cargo.toml'
$previousFlags = $env:CARGO_ENCODED_RUSTFLAGS
$previousRustFlags = $env:RUSTFLAGS
$previousTarget = $env:CARGO_TARGET_DIR
$fingerprint = $null
try {
    # Only force portable backends. Auto detects AVX2 and falls back on other CPUs.
    foreach ($backend in @('auto', 'sse2', 'soft')) {
        $env:RUSTFLAGS = $null
        $env:CARGO_ENCODED_RUSTFLAGS = if ($backend -eq 'auto') { '' } else {
            '--cfg=chacha20_backend="' + $backend + '"'
        }
        $env:CARGO_TARGET_DIR = Join-Path $root "src-tauri/target/rng-bench/$backend"
        Write-Output "backend,$backend"
        cargo test --manifest-path $manifest --release --locked
        if ($LASTEXITCODE -ne 0) { throw "Tests failed: $backend" }
        cargo clippy --manifest-path $manifest --all-targets --locked -- -D warnings
        if ($LASTEXITCODE -ne 0) { throw "Clippy failed: $backend" }
        $result = @(cargo run --manifest-path $manifest --release --locked)
        if ($LASTEXITCODE -ne 0) { throw "Benchmark failed: $backend" }
        $result | Write-Output
        $current = @($result | Where-Object { $_ -like 'fingerprint,*' })
        if ($current.Count -ne 1) { throw 'Missing stream fingerprint' }
        if ($null -eq $fingerprint) { $fingerprint = $current[0] }
        elseif ($fingerprint -ne $current[0]) { throw "Stream mismatch: $backend" }
    }
} finally {
    $env:CARGO_ENCODED_RUSTFLAGS = $previousFlags
    $env:RUSTFLAGS = $previousRustFlags
    $env:CARGO_TARGET_DIR = $previousTarget
}
