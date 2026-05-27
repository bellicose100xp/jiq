use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process;

use chrono::Local;

#[derive(Debug)]
pub enum SaveError {
    Io(io::Error),
    BadPath(String),
    EnvVarMissing(String),
}

impl fmt::Display for SaveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "{}", e),
            Self::BadPath(msg) => write!(f, "{}", msg),
            Self::EnvVarMissing(name) => write!(f, "env var {} not set", name),
        }
    }
}

impl std::error::Error for SaveError {}

impl From<io::Error> for SaveError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub fn ext_for_result() -> &'static str {
    "json"
}

pub fn current_timestamp() -> String {
    format_timestamp(Local::now())
}

pub fn format_timestamp(now: chrono::DateTime<Local>) -> String {
    now.format("%Y%m%d-%H%M%S").to_string()
}

pub fn expand_path(pattern: &str, ext: &str, timestamp: &str) -> Result<PathBuf, SaveError> {
    // Trim surrounding whitespace so a stray space doesn't change the path.
    let pattern = pattern.trim();
    if pattern.is_empty() {
        return Err(SaveError::BadPath("filename is empty".into()));
    }

    let cwd =
        env::current_dir().map_err(|e| SaveError::BadPath(format!("cannot read cwd: {}", e)))?;
    let cwd_str = cwd.to_string_lossy().to_string();

    let mut s = pattern
        .replace("{timestamp}", timestamp)
        .replace("{ext}", ext)
        .replace("{cwd}", &cwd_str);

    s = expand_env_vars(&s)?;
    s = expand_tilde(&s)?;

    if s.trim().is_empty() {
        return Err(SaveError::BadPath("expanded path is empty".into()));
    }

    let path = PathBuf::from(s);
    // Always return an absolute path so the preview shows the full path the
    // user is committing to, not just the bare filename they typed.
    let absolute = if path.is_absolute() {
        path
    } else {
        cwd.join(path)
    };
    Ok(absolute)
}

fn expand_env_vars(input: &str) -> Result<String, SaveError> {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'$' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            if next == b'{' {
                if let Some(end) = input[i + 2..].find('}') {
                    let name = &input[i + 2..i + 2 + end];
                    out.push_str(&lookup_env(name)?);
                    i += 2 + end + 1;
                    continue;
                }
                return Err(SaveError::BadPath(format!(
                    "unclosed ${{ in pattern: {}",
                    input
                )));
            }
            if is_env_var_start(next) {
                let mut j = i + 1;
                while j < bytes.len() && is_env_var_continue(bytes[j]) {
                    j += 1;
                }
                let name = &input[i + 1..j];
                out.push_str(&lookup_env(name)?);
                i = j;
                continue;
            }
        }
        out.push(b as char);
        i += 1;
    }
    Ok(out)
}

fn is_env_var_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_env_var_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn lookup_env(name: &str) -> Result<String, SaveError> {
    env::var(name).map_err(|_| SaveError::EnvVarMissing(name.to_string()))
}

fn expand_tilde(input: &str) -> Result<String, SaveError> {
    if input == "~" {
        return home_dir_string();
    }
    if let Some(rest) = input.strip_prefix("~/") {
        let mut out = home_dir_string()?;
        out.push('/');
        out.push_str(rest);
        return Ok(out);
    }
    Ok(input.to_string())
}

fn home_dir_string() -> Result<String, SaveError> {
    dirs::home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .ok_or_else(|| SaveError::BadPath("cannot determine home directory".into()))
}

pub fn write_atomic(path: &Path, contents: &str) -> Result<PathBuf, SaveError> {
    let parent = path
        .parent()
        .ok_or_else(|| SaveError::BadPath(format!("path has no parent directory: {:?}", path)))?;

    if !parent.as_os_str().is_empty() && !parent.exists() {
        return Err(SaveError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("parent directory does not exist: {}", parent.display()),
        )));
    }

    let parent_for_tmp = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };

    let file_name = path
        .file_name()
        .ok_or_else(|| SaveError::BadPath(format!("path has no file name: {:?}", path)))?
        .to_string_lossy()
        .to_string();

    let tmp_name = format!(".{}.tmp-{}", file_name, process::id());
    let tmp_path = parent_for_tmp.join(&tmp_name);

    let result = (|| -> io::Result<()> {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        drop(file);
        Ok(())
    })();

    if let Err(e) = result {
        let _ = fs::remove_file(&tmp_path);
        return Err(SaveError::Io(e));
    }

    if let Err(rename_err) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        if rename_err.raw_os_error() == Some(libc_exdev())
            || matches!(rename_err.kind(), io::ErrorKind::CrossesDevices)
        {
            fs::write(path, contents).map_err(SaveError::Io)?;
        } else {
            return Err(SaveError::Io(rename_err));
        }
    }

    let canonical = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    Ok(canonical)
}

#[cfg(unix)]
fn libc_exdev() -> i32 {
    18
}

#[cfg(not(unix))]
fn libc_exdev() -> i32 {
    -1
}

#[cfg(test)]
#[path = "save_io_tests.rs"]
mod save_io_tests;
