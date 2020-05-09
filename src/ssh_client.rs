use ssh2::Session;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

pub struct SSHClient {
    ip: String,
    private_key: PathBuf,
}

impl SSHClient {
    pub fn new(ip: String, private_key: PathBuf) -> SSHClient {
        SSHClient { ip, private_key }
    }

    pub fn send_file(&self, local_file: &Path, remote_filename: String) {
        let mut file_contents = Vec::new();
        let mut archive_file = File::open(local_file).unwrap();
        let size = archive_file.read_to_end(&mut file_contents).unwrap();
        let session = self.create_session().unwrap();
        let mut remote_file = session
            .scp_send(Path::new(&remote_filename), 0o777, size as u64, None)
            .unwrap();
        remote_file.write_all(&file_contents).unwrap();
    }

    pub fn run_command(&self, command: String) -> Result<bool, ssh2::Error> {
        let session = self.create_session().unwrap();
        let mut channel = session.channel_session().unwrap();
        channel.exec(&command).unwrap();
        let mut s = String::new();
        channel.read_to_string(&mut s).unwrap();
        channel.wait_close().unwrap();
        channel.exit_status().map(|exit_code| exit_code == 0)
    }

    fn create_session(&self) -> Option<Session> {
        let tcp = TcpStream::connect(self.ip.clone() + ":22").unwrap();
        let mut session = Session::new().unwrap();
        session.set_tcp_stream(tcp);
        session.handshake().unwrap();
        session
            .userauth_pubkey_file("ec2-user", None, &self.private_key, None)
            .unwrap();

        Some(session)
    }
}
