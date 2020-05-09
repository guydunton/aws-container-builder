use ssh2::Session;
use std::convert::From;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

pub struct SSHClient {
    ip: String,
    private_key: PathBuf,
}

#[derive(Debug)]
pub enum SSHClientError {
    IOError(String),
    SSHError(String),
}

impl From<ssh2::Error> for SSHClientError {
    fn from(error: ssh2::Error) -> Self {
        SSHClientError::SSHError(error.to_string())
    }
}

impl From<std::io::Error> for SSHClientError {
    fn from(error: std::io::Error) -> Self {
        SSHClientError::IOError(error.to_string())
    }
}

impl SSHClient {
    pub fn new(ip: String, private_key: PathBuf) -> SSHClient {
        SSHClient { ip, private_key }
    }

    pub fn send_file(
        &self,
        local_file: &Path,
        remote_filename: String,
    ) -> Result<(), SSHClientError> {
        let mut file_contents = Vec::new();
        let mut archive_file = File::open(local_file)?;
        let size = archive_file.read_to_end(&mut file_contents)?;
        let session = self.create_session()?;
        let mut remote_file =
            session.scp_send(Path::new(&remote_filename), 0o777, size as u64, None)?;
        remote_file.write_all(&file_contents)?;

        Ok(())
    }

    pub fn run_command(&self, command: String) -> Result<bool, SSHClientError> {
        let session = self.create_session()?;
        let mut channel = session.channel_session()?;
        channel.exec(&command)?;
        let mut s = String::new();
        channel.read_to_string(&mut s)?;
        channel.wait_close()?;
        let is_ok = channel.exit_status()? == 0;
        Ok(is_ok)
    }

    fn create_session(&self) -> Result<Session, SSHClientError> {
        let tcp = TcpStream::connect(self.ip.clone() + ":22")?;
        let mut session = Session::new()?;
        session.set_tcp_stream(tcp);
        session.handshake()?;
        session.userauth_pubkey_file("ec2-user", None, &self.private_key, None)?;

        Ok(session)
    }
}
