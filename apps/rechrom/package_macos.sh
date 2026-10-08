#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
if [[ "$(uname -s)" != Darwin ]]; then
  echo 'This packaging helper requires macOS.' >&2
  exit 1
fi
profile=debug
for argument in "$@"; do
  [[ "$argument" == "--release" ]] && profile=release
done
cargo build -p rechrom_app "$@"
app_dir="$PWD/target/Rechrom.app"
mkdir -p "$app_dir/Contents/MacOS"
cp "target/$profile/rechrom" "$app_dir/Contents/MacOS/rechrom"
cat > "$app_dir/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>dev.rechrom.browser</string>
<key>CFBundleName</key><string>Rechrom</string>
<key>CFBundleDisplayName</key><string>Rechrom</string>
<key>CFBundleExecutable</key><string>rechrom</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleVersion</key><string>1</string>
<key>CFBundleShortVersionString</key><string>0.0.1</string>
<key>NSHighResolutionCapable</key><true/>
</dict></plist>
PLIST
# Replacing the executable or Info.plist invalidates the linker's ad-hoc
# signature. Sign the completed bundle, matching Chromium's mac packaging
# order, so direct launches never stall in dyld validation.
codesign --force --deep --sign - "$app_dir"
codesign --verify --deep --strict "$app_dir"
printf '%s\n' "$app_dir"
