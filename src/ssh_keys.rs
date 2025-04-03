use std::process::Command;
use std::io;
use std::fs;

/// Generates an SSH key pair for the given repo name.
/// The keys are saved to files with a prefix of `deploy_key_<repo_name>`.
/// Returns a tuple of (private_key_contents, public_key_contents).
pub fn generate_key_pair(repo_name: &str) -> io::Result<(String, String)> {
    let file_prefix = format!("deploy_key_{}", repo_name);
    // Generate the key pair (private and public)
    let status = Command::new("ssh-keygen")
        .arg("-t")
        .arg("ed25519")
        .arg("-f")
        .arg(&file_prefix)
        .arg("-N")
        .arg("")
        .arg("-q")
        .status()?;
    if !status.success() {
        return Err(io::Error::new(io::ErrorKind::Other, "ssh-keygen failed"));
    }
    // Read the generated private key
    let private_key = fs::read_to_string(&file_prefix)?;
    // Read the generated public key
    let public_key = fs::read_to_string(format!("{}.pub", file_prefix))?;
    Ok((private_key, public_key))
}
