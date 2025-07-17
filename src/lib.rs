#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod proto;
pub mod builder;

pub use proto::Command;
pub use chrono::naive::NaiveDateTime;
pub use builder::CommandBuilder;

use std::{collections::{HashSet, VecDeque}, io::{self, BufRead, BufReader, Read, Write}};

use thiserror::Error;

use crate::proto::{ErrResponse, IntoTTP, OkResponse, PublishToken, Response, Value};

/// Follows an active Tesira Text Protocol session
pub struct TesiraSession<R: Read, W: Write> {
    read_stream: BufReader<R>,
    write_stream: W,
    pending_token: VecDeque<PublishToken>
}

#[cfg(feature = "ssh")]
struct SshPassword(String);

#[cfg(feature = "ssh")]
impl ssh2::KeyboardInteractivePrompt for SshPassword {
    fn prompt<'a>(
            &mut self,
            _username: &str,
            _instructions: &str,
            _prompts: &[ssh2::Prompt<'a>],
        ) -> Vec<String> {
        return vec![self.0.to_owned()];
    }
}

#[cfg(feature = "ssh")]
impl TesiraSession<ssh2::Channel, ssh2::Channel> {

    /// Connect to tesira device over SSH
    pub fn new_from_ssh(hostname: String, username: String, password: String) -> Result<Self, Error> {
        let connection = std::net::TcpStream::connect(hostname.as_str())?;

        let mut ssh = ssh2::Session::new()?;
        ssh.set_tcp_stream(connection);
        ssh.handshake()?;
        ssh.userauth_keyboard_interactive(&username, &mut SshPassword(password))?;

        Self::new_from_ssh_session(&ssh)
    }

    /// Connect to tesira from an **established** and **authenticated** ssh session
    /// It will create a new channel to communicate with device
    pub fn new_from_ssh_session(session: &ssh2::Session) -> Result<Self, Error> {
        let mut channel = session.channel_session()?;
        channel.request_pty("ansi", None, None)?;
        channel.shell()?;
        Self::new_from_stream(channel.clone(), channel)
    }
}

impl<R:Read, W: Write> TesiraSession<R, W> {
    /// Create a new session from arbitrary read and write stream
    /// 
    /// See [TesiraSession::new_from_ssh] to use ssh
    pub fn new_from_stream(read_strea: R, write_stream: W) -> Result<Self, Error> {
        let mut new_self = Self {
            read_stream: BufReader::new(read_strea),
            write_stream,
            pending_token: VecDeque::new()
        };
        let mut banner_buffer = String::new();
        while !banner_buffer.starts_with("Welcome") { // Wait for welcome line
            banner_buffer.clear();
            new_self.read_stream.read_line(&mut banner_buffer)?;
        }
        Ok(new_self)
    }

    /// Get all available aliases 
    pub fn get_aliases(&mut self) -> Result<HashSet<String>, Error> {
        let response = self.send_command(Command::builder().session().aliases())?;
        if let OkResponse::WithList(l) = response {
            return Ok(l.into_iter().filter_map(|it| {
                match it {
                    Value::String(v) => Some(v),
                    _ => None
                }
            }).collect::<HashSet<_>>())
        } else {
            return Err(Error::UnexpectedResponse(Response::Ok(response), "a response with a list of aliases".to_owned()))
        }
    }

    /// Send direct command and await for a response from device
    /// 
    /// See [TesiraSession::set], [TesiraSession::get], [TesiraSession::get_aliases] or [TesiraSession::subscribe]
    pub fn send_command<'a, 'b:'a>(&'a mut self, cmd: impl Into<Command<'b>>) -> Result<OkResponse, Error> {
        let command: Command = cmd.into();
        let cmd_str = format!("{}\n", command.into_ttp());
        self.write_stream.write_all(&cmd_str.as_bytes())?;
        loop {
            let response = self.recv_response()?;
            match response {
                Response::Err(e) => return Err(Error::OperationFailed(e)),
                Response::Ok(res) => return Ok(res),
                Response::PublishToken(t) => self.pending_token.push_front(t),
            }
        }
    }

    fn recv_response(&mut self) -> Result<Response, Error> {
        let mut buf = String::new();
        loop { // Ignore empty lines
            let byte_red = self.read_stream.read_line(&mut buf)?;
            if byte_red == 0 {
                return Err(Error::UnexpectedEnd);
            }

            let trim_buf = buf.trim();
            if !trim_buf.is_empty() && (&trim_buf[0..1] == "-" || &trim_buf[0..1] == "+" || &trim_buf[0..1] == "!") {
                return Ok(Response::parse_ttp(&buf)?);
            } else {
                buf.clear();
            }
        }
    }

    /// Await for a publish token to come
    /// 
    /// Please prefer usage of [TesiraSession::subscribe] and [TesiraSession::dispatch_next_token]
    /// and use channels to receive PublishToken in a multithreaded environment
    /// 
    /// Use this method if you subscribed manually and wants to get all Publish tokens in one thread
    pub fn recv_token(&mut self) -> Result<PublishToken, Error> {
        if let Some(pending_token) = self.pending_token.pop_back() {
            return Ok(pending_token);
        }
        
        let response = self.recv_response()?;
        match response {
            Response::PublishToken(t) => 
                Ok(t),
            r @ ( Response::Err(_) | Response::Ok(_) ) => 
                Err(Error::UnexpectedResponse(r, "a publish token".to_owned())),
        }
    }
}

/// Error that can occur when interacting with Tesira sessions
#[derive(Debug, Error)]
pub enum Error {
    /// IO Error on streams
    #[error("IO Error : {0}")]
    IO(#[from] io::Error),
    /// Received an Error response
    #[error("Operation failed on device : {0}")]
    OperationFailed(ErrResponse),
    /// Failed to parse response send by device
    #[error("Response parsing failed : {0}")]
    ParsingFailed(String),
    /// Response sent by device wasn't expected
    #[error("Unexpected response from device: {0:?} (expected {1})")]
    UnexpectedResponse(Response, String),
    /// Stream ends before end of response
    #[error("Unexpected end of read stream")]
    UnexpectedEnd,
    #[cfg(feature = "ssh")]
    #[error("SSH error: {0}")]
    /// SSH error
    Ssh(#[from] ssh2::Error)
}

impl<'a> From<proto::Error<'a>> for Error {
    fn from(value: proto::Error) -> Self {
        Self::ParsingFailed(format!("{}", value))
    }
}

mod test {
    #[allow(unused_imports)]
    use std::{cell::LazyCell, collections::HashSet, io::{BufReader, BufWriter, Cursor, Write}};

    #[allow(unused_imports)]
    use crate::{proto::{Command, ErrResponse, OkResponse, PublishToken, Value}, Error, TesiraSession};
    
    #[allow(dead_code)]
    const WELCOME_BANNER: LazyCell<Vec<u8>> = LazyCell::new(|| "Welcome to the Tesira Text Protocol Server...\n\n".as_bytes().to_vec());

    #[test]
    fn should_handle_valid_set_command(){
        let write_c = Cursor::new(Vec::new());
        let read_c = Cursor::new(WELCOME_BANNER.clone());
        let mut session = TesiraSession::new_from_stream(read_c, write_c)
            .unwrap();

        session.read_stream.get_mut().get_mut().extend_from_slice("Level3 set level 2 0\n".as_bytes()); // Should also handle echo
        session.read_stream.get_mut().get_mut().extend_from_slice("+OK\n".as_bytes());
        session.send_command(Command::new_set("Level3", "level", [2], 0)).unwrap();

        assert_eq!(session.write_stream.into_inner(), "Level3 set level 2 0\n".as_bytes().to_vec());
    }

    #[test]
    fn should_handle_valid_get_command(){
        let write_c = Cursor::new(Vec::new());
        let read_c = Cursor::new(WELCOME_BANNER.clone());

        let mut session = TesiraSession::new_from_stream(read_c, write_c)
            .unwrap();
        
        session.read_stream.get_mut().get_mut().extend_from_slice("Level3 get level 2\n".as_bytes()); // Should also handle echo
        session.read_stream.get_mut().get_mut().extend_from_slice("+OK \"value\":0.000000\n".as_bytes());
        let response = session.send_command(Command::new_get("Level3", "level", [2])).unwrap();
        
        assert_eq!(session.write_stream.into_inner(), "Level3 get level 2\n".as_bytes().to_vec());
        assert_eq!(response, OkResponse::WithValue(Value::Number(0.0)));
    }

    #[test]
    fn should_handle_valid_get_aliases_command(){
        let write_c = Cursor::new(Vec::new());
        let read_c = Cursor::new(WELCOME_BANNER.clone());

        let mut session = TesiraSession::new_from_stream(read_c, write_c)
            .unwrap();
        
        session.read_stream.get_mut().get_mut().extend_from_slice("SESSION get aliases\n".as_bytes()); // Should also handle echo
        session.read_stream.get_mut().get_mut().extend_from_slice("+OK \"list\":[\"AecInput1\" \"AudioMeter2\" \"AudioMeter4\" \"DEVICE\" \"DanteInput1\" \"DanteOutput1\" \"Level1\" \"Level2\" \"Level3\" \"Mixer1\" \"NoiseGenerator1\" \"Output1\" \"Router1\" \"ToneGenerator1\" \"ToneGenerator2\" \"USBInput1\" \"USBOutput1\"]\n".as_bytes());
        let response = session.get_aliases().unwrap();
        
        assert_eq!(session.write_stream.into_inner(), "SESSION get aliases\n".as_bytes().to_vec());
        assert_eq!(
            response,
            HashSet::from([
                "AecInput1".to_owned(),
                "AudioMeter2".to_owned(),
                "AudioMeter4".to_owned(),
                "DEVICE".to_owned(),
                "DanteInput1".to_owned(),
                "DanteOutput1".to_owned(),
                "Level1".to_owned(),
                "Level2".to_owned(),
                "Level3".to_owned(),
                "Mixer1".to_owned(),
                "NoiseGenerator1".to_owned(),
                "Output1".to_owned(),
                "Router1".to_owned(),
                "ToneGenerator1".to_owned(),
                "ToneGenerator2".to_owned(),
                "USBInput1".to_owned(),
                "USBOutput1".to_owned()
                ])
            );
    }

    #[test]
    fn should_handle_failed_operation(){
        let write_c = Cursor::new(Vec::new());
        let read_c = Cursor::new(WELCOME_BANNER.clone());

        let mut session = TesiraSession::new_from_stream(read_c, write_c)
            .unwrap();
        
        session.read_stream.get_mut().get_mut().extend_from_slice("Level3 set mute 3 true\n".as_bytes()); // Should also handle echo
        session.read_stream.get_mut().get_mut().extend_from_slice("-ERR address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}\n".as_bytes());
        let response = session.send_command(Command::new_set("Level3", "mute", [3], true));

        assert_eq!(session.write_stream.into_inner(), "Level3 set mute 3 true\n".as_bytes().to_vec());
        
        if let Err(Error::OperationFailed(e)) = response {
            assert_eq!(e, ErrResponse{
                message: "address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}".to_owned()
            })
        } else {
            panic!("Unexpected response : {:?}", response)
        }
    }

    #[test]
    fn should_handle_subscription(){
        let write_c = Cursor::new(Vec::new());
        let read_c = Cursor::new(WELCOME_BANNER.clone());

        let mut session = TesiraSession::new_from_stream(read_c, write_c)
            .unwrap();
        
        session.read_stream.get_mut().get_mut().extend_from_slice("LogicMeter1 subscribe state 1 Subscription0\n".as_bytes());
        session.read_stream.get_mut().get_mut().extend_from_slice("! \"publishToken\":\"Subscription0\" \"value\":false\n".as_bytes());
        session.read_stream.get_mut().get_mut().extend_from_slice("+OK\n".as_bytes());
        let receiver = session.send_command(Command::new_subscribe("LogicMeter1", "state", [1], "Subscription0")).unwrap();

        assert_eq!(*session.write_stream.get_ref(), "LogicMeter1 subscribe state 1 Subscription0\n".as_bytes().to_vec());

        assert_eq!(session.recv_token().unwrap(), PublishToken {
            label: "Subscription0".to_owned(),
            value: Value::Boolean(false)
        });

        session.read_stream.get_mut().get_mut().extend_from_slice("! \"publishToken\":\"Subscription0\" \"value\":true\n".as_bytes());
        assert_eq!(session.recv_token().unwrap(), PublishToken {
            label: "Subscription0".to_owned(),
            value: Value::Boolean(true)
        });

        session.read_stream.get_mut().get_mut().extend_from_slice("! \"publishToken\":\"Subscription0\" \"value\":false\n".as_bytes());
        assert_eq!(session.recv_token().unwrap(), PublishToken {
            label: "Subscription0".to_owned(),
            value: Value::Boolean(false)
        });
    }
}