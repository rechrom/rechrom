use serde_json::json;
use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

pub struct CrashReport<'a> {
    pub source: &'a str,
    pub message: &'a str,
    pub url: &'a str,
    pub working_directory: &'a Path,
}

pub struct StoredCrashReport {
    pub archived_path: PathBuf,
    pub latest_path: PathBuf,
}

/// Storage boundary owned by the embedding host. It deliberately has no
/// knowledge of windows, browser engines, or platform-specific log folders.
pub trait CrashReportSink {
    fn write(&self, report: CrashReport<'_>) -> io::Result<StoredCrashReport>;
}

pub struct JsonCrashReportSink {
    directory: PathBuf,
}

impl JsonCrashReportSink {
    pub fn new(directory: PathBuf) -> Self {
        Self { directory }
    }
}

impl CrashReportSink for JsonCrashReportSink {
    fn write(&self, report: CrashReport<'_>) -> io::Result<StoredCrashReport> {
        fs::create_dir_all(&self.directory)?;
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let timestamp_ms = elapsed.as_millis();
        let timestamp_ns = elapsed.as_nanos();
        let pid = std::process::id();
        let bytes = serde_json::to_vec_pretty(&json!({
            "schema_version": 1,
            "timestamp_unix_ms": timestamp_ms,
            "pid": pid,
            "version": env!("CARGO_PKG_VERSION"),
            "source": report.source,
            "message": report.message,
            "url": report.url,
            "working_directory": report.working_directory,
        }))
        .map_err(io::Error::other)?;

        let archived_path = self.directory.join(format!("{timestamp_ns}-{pid}.json"));
        write_new(&archived_path, &bytes)?;

        // The stable filename is the automation/debugging API. Replace it
        // only after the complete archive exists, so a failed latest update
        // never destroys the primary report.
        let latest_path = self.directory.join("latest.json");
        let latest_staging = self.directory.join(format!(".latest-{pid}.tmp"));
        write_replace(&latest_staging, &bytes)?;
        if let Err(error) = fs::rename(&latest_staging, &latest_path) {
            // Windows does not replace an existing destination with rename.
            if latest_path.exists() {
                fs::remove_file(&latest_path)?;
                fs::rename(&latest_staging, &latest_path)?;
            } else {
                return Err(error);
            }
        }
        Ok(StoredCrashReport {
            archived_path,
            latest_path,
        })
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn write_replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}
