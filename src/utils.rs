use std::ffi::OsStr;
use std::io;
use std::process::{Output, Stdio};

use base64::engine::general_purpose::STANDARD;
use base64::{DecodeError, Engine};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub fn validate_key(input: &str) -> Result<(), DecodeError> {
    let regex = regex::Regex::new(r"^[A-Za-z0-9+/]{43}=$").unwrap();

    if !regex.is_match(input) {
        return Err(DecodeError::InvalidByte(0, 0));
    }

    STANDARD.decode(input)?;

    Ok(())
}

pub fn validate_identifier(input: &str) -> bool {
    let regex = regex::Regex::new(r"^[A-Z]+-[A-Z]+-[A-Za-z0-9]{1,32}$").unwrap();

    regex.is_match(input)
}

pub async fn exec<S: AsRef<OsStr>, B: AsRef<[u8]>>(
    command: S,
    args: &[S],
    stdin: Option<B>,
) -> io::Result<Output> {
    let mut cmd = Command::new(command);

    for arg in args {
        cmd.arg(arg);
    }

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let mut child = cmd.spawn()?;

    if let Some(stdin) = stdin {
        let stdin_stream = child.stdin.as_mut().expect("Failed to open stdin");
        stdin_stream.write_all(stdin.as_ref()).await?;
    }

    child.wait_with_output().await
}

pub async fn apply_vyatta_cfg<B: AsRef<[u8]>>(cfg: B) -> io::Result<Vec<u8>> {
    let output = exec("vbash", &["-s"], Some(cfg)).await?;
    Ok(output.stdout)
}
