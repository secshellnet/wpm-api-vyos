use std::ffi::OsStr;
use std::io;
use std::process::{Output, Stdio};

use base64::DecodeError;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

pub fn validate_key(input: &str) -> Result<(), DecodeError> {
    let regex = regex::Regex::new(r"^[A-Za-z0-9+/]{43}=$").unwrap();

    if !regex.is_match(input) {
        return Err(DecodeError::InvalidByte(0, 0));
    }

    let decoded = match base64::decode(input) {
        Ok(_) => Ok(()),
        Err(err) => Err(err),
    };

    decoded
}

pub async fn exec<S: AsRef<OsStr>, B: AsRef<[u8]>>(command: S, args: &[S], stdin: Option<B>) -> io::Result<Output> {
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

pub async fn apply_vyatta_cfg<B: AsRef<[u8]>>(cfg: B) -> io::Result<bool>{
    let output = exec("vbash", &["-s"], Some(cfg)).await?;

    // valid means is configured
    // when a peer has been marked for deletion
    //   valid=false indicates that a peer has been deleted
    // when a peer have just been created:
    //   valid=true indicates that the peer has been created successfully
    let valid = &output.stdout.len() > &1;

    Ok(valid)
}
