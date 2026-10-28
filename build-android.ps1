# Build in an isolated container copy: modern host Cargo may rewrite lock v3 to v4.
$ErrorActionPreference = 'Stop'
$projectRoot = $PSScriptRoot
$buildTarget = Join-Path $projectRoot 'target'
New-Item -ItemType Directory -Force -Path $buildTarget | Out-Null
docker run --rm --mount "type=volume,source=ricochet-android-signing,target=/root/.android" --mount "type=volume,source=ricochet-android-registry,target=/usr/local/cargo/registry" -e CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse --mount "type=bind,source=$projectRoot,target=/project,readonly" --mount "type=bind,source=$buildTarget,target=/root/src/target" -w /root/src notfl3/cargo-apk sh /project/build-android.sh
if ($LASTEXITCODE -ne 0) { throw 'Android build failed; see Docker output above.' }
Write-Host "APK directory: $buildTarget\android-artifacts\release\apk"



