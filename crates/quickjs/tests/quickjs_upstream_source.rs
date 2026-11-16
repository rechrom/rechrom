// Expression shared by the C differential oracles: the directory of the
// official, unmodified QuickJS release recorded in crates/quickjs/UPSTREAM.json.
// The release is not stored in the repository. It is downloaded once into the
// Cargo target directory, verified against the pinned SHA-256 and reused.
// Set RECHROM_QUICKJS_UPSTREAM_DIR to use an already extracted copy.
{
    const RELEASE: &str = "quickjs-2026-06-04";
    const URL: &str = "https://bellard.org/quickjs/quickjs-2026-06-04.tar.xz";
    const SHA256: &str = "b376e839b322978313d929fd20663b11ba58b75df5a46c126dd19ea2fa70ad2a";
    let manifest = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(dir) = std::env::var_os("RECHROM_QUICKJS_UPSTREAM_DIR") {
        std::path::PathBuf::from(dir)
    } else {
        let cache = std::env::var_os("CARGO_TARGET_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| manifest.join("../../target"))
            .join("quickjs-upstream");
        let source = cache.join(RELEASE);
        if !source.join("quickjs.c").exists() {
            // Each process stages privately, then renames into place, so
            // concurrently running test binaries never observe a partial tree.
            let staging = cache.join(format!(
                "staging-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(&staging).unwrap();
            let archive = "quickjs.tar.xz";
            let status = std::process::Command::new("curl")
                .args(["-sSfL", "-o", archive, URL])
                .current_dir(&staging)
                .status()
                .expect("curl is required to fetch the QuickJS oracle source");
            assert!(status.success(), "failed to download {URL}");
            let bytes = std::fs::read(staging.join(archive)).unwrap();
            let digest = <sha2::Sha256 as sha2::Digest>::digest(&bytes);
            let digest = digest
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>();
            assert_eq!(digest, SHA256, "{URL} does not match the pinned release");
            // Relative paths only: GNU tar treats `C:` in a path as a host.
            let status = std::process::Command::new("tar")
                .args(["-xJf", archive])
                .current_dir(&staging)
                .status()
                .expect("tar with xz support is required to unpack the QuickJS oracle");
            assert!(status.success(), "failed to unpack {archive}");
            if std::fs::rename(staging.join(RELEASE), &source).is_err() {
                assert!(
                    source.join("quickjs.c").exists(),
                    "could not install {}",
                    source.display()
                );
            }
            let _ = std::fs::remove_dir_all(&staging);
        }
        source
    }
}
