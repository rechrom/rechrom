#!/usr/bin/env bash
# Regenerate vendor/patches/<crate>.patch: the exact difference between each
# vendored crate and its pristine upstream package (Chromium keeps local
# crate changes the same way, in third_party/rust/chromium_crates_io/patches).
#
# Usage: tools/vendor_patches.sh <pristine-root>
#   <pristine-root>/<crate>/ must hold the extracted upstream source (the
#   crates.io package, or the pinned GitHub archive for crabby-avif) and the
#   downloaded archive (pkg.crate / src.tar.gz) for its SHA-256.
# Files present only upstream (examples, tests, CI) are listed, not diffed.
# Rechrom's own note files are excluded from the diff.
set -euo pipefail
root=$(cd "$(dirname "$0")/.." && pwd)
pristine=${1:?usage: tools/vendor_patches.sh <pristine-root>}
out="$root/vendor/patches"
mkdir -p "$out"
notes='^(RECHROM.*|LOCAL_CHANGES\.md|README\.chromium\.md|UPSTREAM.*|DECISIONS\.md|Cargo\.toml\.orig|\.cargo_vcs_info\.json|.*\.upstream)$'

declare -A src=(
  [crabby-avif]="crabby-avif/CrabbyAvif-8f2b74b4d8be956395f527020c2190762e086f98|crabby-avif/src.tar.gz|https://github.com/webmproject/CrabbyAvif/archive/8f2b74b4d8be956395f527020c2190762e086f98.tar.gz"
  [hyper-util]="hyper-util/hyper-util-0.1.21|hyper-util/pkg.crate|https://static.crates.io/crates/hyper-util/hyper-util-0.1.21.crate"
  [rav1d]="rav1d/rav1d-1.1.0|rav1d/pkg.crate|https://static.crates.io/crates/rav1d/rav1d-1.1.0.crate"
  [sfv]="sfv/sfv-0.15.0|sfv/pkg.crate|https://static.crates.io/crates/sfv/sfv-0.15.0.crate"
  [smithay-clipboard]="smithay-clipboard/smithay-clipboard-0.7.2|smithay-clipboard/pkg.crate|https://static.crates.io/crates/smithay-clipboard/smithay-clipboard-0.7.2.crate"
  [softbuffer]="softbuffer/softbuffer-0.4.8|softbuffer/pkg.crate|https://static.crates.io/crates/softbuffer/softbuffer-0.4.8.crate"
  [winit]="winit/winit-0.30.13|winit/pkg.crate|https://static.crates.io/crates/winit/winit-0.30.13.crate"
  [xml]="xml/xml-1.4.0|xml/pkg.crate|https://static.crates.io/crates/xml/xml-1.4.0.crate"
  [zune-jpeg]="zune-jpeg/zune-jpeg-0.5.15|zune-jpeg/pkg.crate|https://static.crates.io/crates/zune-jpeg/zune-jpeg-0.5.15.crate"
)

for crate in $(printf '%s\n' "${!src[@]}" | sort); do
  IFS='|' read -r up archive url <<<"${src[$crate]}"
  up="$pristine/$up"; vend="$root/vendor/$crate"; patch="$out/$crate.patch"
  [ -d "$up" ] && [ -d "$vend" ] || { echo "skip $crate (missing $up or $vend)" >&2; continue; }
  sha=$(sha256sum "$pristine/$archive" | cut -d' ' -f1)
  {
    echo "# vendor/$crate against pristine upstream"
    echo "# upstream: $url"
    echo "# sha256:   $sha"
    echo "# Regenerate with tools/vendor_patches.sh; see vendor/DECISIONS.md for why it is kept."
    echo "#"
    echo "# Upstream files not vendored:"
    (cd "$up" && find . -type f | sed 's#^\./##' | sort) | while read -r f; do
      base=${f##*/}; [[ $base =~ $notes ]] && continue
      [ -e "$vend/$f" ] || echo "#   $f"
    done
    (cd "$vend" && find . -type f | sed 's#^\./##' | sort) | while read -r f; do
      base=${f##*/}; [[ $base =~ $notes ]] && continue
      [ "${f#target/}" != "$f" ] && continue
      if [ -e "$up/$f" ]; then
        diff -u --strip-trailing-cr --label "a/$f" --label "b/$f" "$up/$f" "$vend/$f" || true
      else
        diff -u --strip-trailing-cr --label /dev/null --label "b/$f" /dev/null "$vend/$f" || true
      fi
    done
  } > "$patch"
  echo "$crate: $(grep -c '^+[^+]' "$patch") added, $(grep -c '^-[^-]' "$patch") removed lines, $(grep -c '^#   ' "$patch") upstream files not vendored"
done
