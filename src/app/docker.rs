use std::{io, path::Path, process::Command};

use super::progress::{ScanProgress, ScanProgressDisplay};

pub(super) struct DockerCleanup {
    pub(super) reclaimable_size: u64,
}

pub(super) fn scan(display: ScanProgressDisplay) -> io::Result<Option<DockerCleanup>> {
    let mut progress = ScanProgress::new(3, display);
    progress.update(Path::new("docker system df"));

    let output = match Command::new("docker").args(["system", "df", "--format", "{{.Type}}\t{{.Reclaimable}}"]).output()
    {
        Ok(output) => output,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            progress.finish();
            return Ok(None);
        }
        Err(error) => return Err(error),
    };
    progress.finish();

    if !output.status.success() {
        return Ok(None);
    }

    let reclaimable_size = String::from_utf8_lossy(&output.stdout).lines().filter_map(reclaimable_size).sum();
    Ok(Some(DockerCleanup { reclaimable_size }))
}

pub(super) fn prune() -> io::Result<()> {
    for arguments in [
        ["container", "prune", "--force"],
        ["image", "prune", "--force"],
        ["network", "prune", "--force"],
        ["builder", "prune", "--force"],
    ] {
        let status = Command::new("docker").args(arguments).status()?;
        if !status.success() {
            return Err(io::Error::from(io::ErrorKind::PermissionDenied));
        }
    }
    Ok(())
}

fn reclaimable_size(line: &str) -> Option<u64> {
    let (kind, value) = line.split_once('\t')?;
    if kind == "Local Volumes" {
        return None;
    }
    parse_size(value.split_once(" (")?.0)
}

fn parse_size(value: &str) -> Option<u64> {
    let unit_start = value.find(|character: char| character.is_ascii_alphabetic())?;
    let number = &value[..unit_start];
    let multiplier = match &value[unit_start..] {
        "B" => 1,
        "kB" => 1_000,
        "MB" => 1_000_000,
        "GB" => 1_000_000_000,
        "TB" => 1_000_000_000_000,
        _ => return None,
    };
    let (whole, fractional) = match number.split_once('.') {
        Some((whole, fractional)) => (whole, fractional),
        None => (number, ""),
    };
    let whole = whole.parse::<u64>().ok()?.checked_mul(multiplier)?;
    let fractional = if fractional.is_empty() {
        0
    } else {
        let scale = 10u64.checked_pow(u32::try_from(fractional.len()).ok()?)?;
        fractional.parse::<u64>().ok()?.checked_mul(multiplier)? / scale
    };
    whole.checked_add(fractional)
}

#[cfg(test)]
mod tests {
    use super::{parse_size, reclaimable_size};

    #[test]
    fn parses_docker_reclaimable_sizes() {
        assert_eq!(parse_size("1.5GB"), Some(1_500_000_000));
        assert_eq!(reclaimable_size("Images\t1.5GB (50%)"), Some(1_500_000_000));
        assert_eq!(reclaimable_size("Local Volumes\t10GB (50%)"), None);
    }
}
