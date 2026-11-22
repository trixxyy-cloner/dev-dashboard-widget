use crate::source::command;
use serde::Serialize;
use std::path::Path;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalStatus {
    pub branch: String,
    pub changed_files: u32,
    pub ahead: u32,
    pub behind: u32,
}

pub async fn status(path: &str) -> Result<LocalStatus, String> {
    if !Path::new(path).exists() {
        return Err("Mappen finns inte".into());
    }
    let output = command("git")
        .args(["-C", path, "status", "--porcelain=v2", "--branch"])
        .output()
        .await
        .map_err(|e| format!("Kunde inte köra git: {e}"))?;
    if !output.status.success() {
        return Err("Inte ett git-repo".into());
    }
    Ok(parse(&String::from_utf8_lossy(&output.stdout)))
}

fn parse(porcelain: &str) -> LocalStatus {
    let mut status = LocalStatus { branch: "?".into(), changed_files: 0, ahead: 0, behind: 0 };
    for line in porcelain.lines() {
        if let Some(head) = line.strip_prefix("# branch.head ") {
            status.branch = head.to_string();
        } else if let Some(ab) = line.strip_prefix("# branch.ab ") {
            // Format: "+<ahead> -<behind>"
            let mut parts = ab.split_whitespace();
            status.ahead = parts.next().and_then(|p| p.trim_start_matches('+').parse().ok()).unwrap_or(0);
            status.behind = parts.next().and_then(|p| p.trim_start_matches('-').parse().ok()).unwrap_or(0);
        } else if !line.starts_with('#') && !line.is_empty() {
            status.changed_files += 1;
        }
    }
    status
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reads_branch_ahead_behind_and_changes() {
        let input = "# branch.oid abc\n# branch.head main\n# branch.upstream origin/main\n# branch.ab +2 -1\n1 .M N... 100644 100644 100644 a b src/a.ts\n? new.txt\n";
        let status = parse(input);
        assert_eq!(status.branch, "main");
        assert_eq!(status.ahead, 2);
        assert_eq!(status.behind, 1);
        assert_eq!(status.changed_files, 2);
    }

    #[test]
    fn parse_without_upstream_has_zero_ahead_behind() {
        let status = parse("# branch.head feature/x\n");
        assert_eq!(status.branch, "feature/x");
        assert_eq!(status.ahead, 0);
        assert_eq!(status.changed_files, 0);
    }
}
