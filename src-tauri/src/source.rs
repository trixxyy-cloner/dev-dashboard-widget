use serde::Serialize;

/// Result of one data source, so a failing API only blanks out its own row.
#[derive(Debug, Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
pub enum Source<T> {
    Ok { data: T },
    Error { message: String },
    Off,
}

impl<T> From<Result<T, String>> for Source<T> {
    fn from(result: Result<T, String>) -> Self {
        match result {
            Ok(data) => Source::Ok { data },
            Err(message) => Source::Error { message },
        }
    }
}

/// Runs a command without flashing a console window on Windows.
pub fn command(program: &str) -> tokio::process::Command {
    #[allow(unused_mut)]
    let mut cmd = tokio::process::Command::new(program);
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

pub async fn check_status(response: reqwest::Response) -> Result<reqwest::Response, String> {
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }
    Err(match status.as_u16() {
        401 | 403 => format!("Ogiltig token ({status})"),
        404 => "Hittades inte (404)".into(),
        _ => format!("HTTP {status}"),
    })
}
