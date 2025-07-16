pub mod parser;
use std::{fmt::Display, time::Duration};
use parser::parse_response;
use thiserror::Error;

#[derive(Debug, Clone)]
pub enum Command {
    GetAliases,
    Set(SetAttributeCommand),
    Get(GetAttributeCommand),
    Increment(IncrementAttributeCommand),
    Decrement(DecrementAttributeCommand),
    Toggle(ToggleAttributeCommand),
    Subscribe(SubscribeCommand),
    Unsubscribe(UnsubscribeCommand)
}

#[derive(Debug, Clone)]
pub struct SetAttributeCommand {
    pub instance_tag: InstanceTag,
    pub attribute: String,
    pub args: Vec<String>
}

#[derive(Debug, Clone)]
pub struct GetAttributeCommand {
    pub instance_tag: InstanceTag,
    pub attribute: String,
    pub args: Vec<String>
}

#[derive(Debug, Clone)]
pub struct IncrementAttributeCommand {
    instance_tag: InstanceTag,
    attribute: String,
    args: Vec<String>
}

#[derive(Debug, Clone)]
pub struct DecrementAttributeCommand {
    pub instance_tag: InstanceTag,
    pub attribute: String,
    pub args: Vec<String>
}

#[derive(Debug, Clone)]
pub struct ToggleAttributeCommand {
    pub instance_tag: InstanceTag,
    pub attribute: String,
    pub args: Vec<String>
}

#[derive(Debug, Clone)]
pub struct SubscribeCommand {
    pub instance_tag: InstanceTag,
    pub attribute: String,
    pub index: Option<i32>,
    pub label: Option<String>,
    pub minimum_rate: Option<Duration>
}

#[derive(Debug, Clone)]
pub struct UnsubscribeCommand {
    pub instance_tag: InstanceTag,
    pub attribute: String,
    pub index: Option<i32>,
    pub label: Option<String>
}

impl From<SubscribeCommand> for UnsubscribeCommand {
    fn from(value: SubscribeCommand) -> Self {
        Self { instance_tag: value.instance_tag, attribute: value.attribute, index: value.index, label: value.label }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    Ok(OkResponse),
    Err(ErrResponse),
    PublishToken(PublishToken)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ErrResponse {
    pub message: String
}

impl Display for ErrResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OkResponse {
    Ok,
    WithValue(Value),
    WithList(Vec<Value>)
}

#[derive(Debug, Clone, PartialEq)]
pub struct PublishToken {
    pub label: String,
    pub value: Vec<Value>
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String)
}

pub type InstanceTag = String;

impl Response {
    pub fn parse_ttp(source: &str) -> Result<Self, Error> {
        parse_response(source)
            .map(|it| it.1)
            .map_err(|e| match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => Error::ParseError(e),
                nom::Err::Incomplete(e) => {
                    println!("STILL NEEDED {e:?}");
                    Error::UnexpectedEnd
                }
            })
    }
}

#[derive(Debug, Error)]
pub enum Error<'a> {
    #[error("Response parse error: {0}")]
    ParseError(nom::error::Error<&'a str>),
    #[error("Unexpected end of input")]
    UnexpectedEnd
}

pub trait IntoTTP {
    fn into_ttp(self) -> String;
}

impl IntoTTP for Command {
    fn into_ttp(self) -> String {
        match self {
            Command::GetAliases => "SESSION get aliases".to_owned(),
            Command::Set(cmd) => format!("{} set {} {}", cmd.instance_tag, cmd.attribute, cmd.args.join(" ")),
            Command::Get(cmd) => format!("{} get {} {}", cmd.instance_tag, cmd.attribute, cmd.args.join(" ")),
            Command::Increment(cmd) => format!("{} increment {} {}", cmd.instance_tag, cmd.attribute, cmd.args.join(" ")),
            Command::Decrement(cmd) => format!("{} decrement {} {}", cmd.instance_tag, cmd.attribute, cmd.args.join(" ")),
            Command::Toggle(cmd) => format!("{} toggle {} {}", cmd.instance_tag, cmd.attribute, cmd.args.join(" ")),
            Command::Subscribe(cmd) => {
                let mut result = format!("{} subscribe {}", cmd.instance_tag, cmd.attribute);
                
                if let Some(index) = cmd.index {
                    result = format!("{result} {}", index);
                }

                if let Some(label) = cmd.label {
                    result = format!("{result} {}", label)
                }

                if let Some(value) = cmd.minimum_rate {
                    result = format!("{result} {}", value.as_millis())
                }

                result
            }
            Command::Unsubscribe(cmd) => {
                let mut result = format!("{} unsubscribe {}", cmd.instance_tag, cmd.attribute);
                
                if let Some(index) = cmd.index {
                    result = format!("{result} {}", index);
                }

                if let Some(label) = cmd.label {
                    result = format!("{result} {}", label)
                }

                result
            }
        }
    }
}

mod test {
    #[allow(unused_imports)]
    use crate::proto::{Command, ErrResponse, GetAttributeCommand, IntoTTP, OkResponse, PublishToken, Response, SetAttributeCommand, Value};

    #[test]
    fn should_serialize_get_alias_command() {
        assert_eq!(Command::GetAliases.into_ttp(), "SESSION get aliases");
    }

    #[test]
    fn should_serialize_get_command() {
        assert_eq!(
            Command::Get(GetAttributeCommand {
                instance_tag: "Level3".to_owned(),
                attribute: "level".to_owned(),
                args: vec!["2".to_owned()]
            }).into_ttp(),
            "Level3 get level 2");
    }

    #[test]
    fn should_serialize_set_command() {
        assert_eq!(
            Command::Set(SetAttributeCommand {
                instance_tag: "level3".to_owned(),
                attribute: "mute".to_owned(),
                args: vec![
                    "3".to_owned(),
                    "true".to_owned()
                ]
            }).into_ttp(),
            "level3 set mute 3 true");

        assert_eq!(
            Command::Set(SetAttributeCommand {
                instance_tag: "level3".to_owned(),
                attribute: "mute".to_owned(),
                args: vec![
                    "0".to_owned(),
                    "true".to_owned()
                ]
            }).into_ttp(),
            "level3 set mute 0 true");
    }

    #[test]
    fn should_parse_simple_ok_response(){
        assert_eq!(Response::parse_ttp("+OK").unwrap(), Response::Ok(OkResponse::Ok));
    }

    #[test]
    fn should_parse_ok_response_with_value() {
        assert_eq!(Response::parse_ttp("+OK \"value\":0.000000").unwrap(), Response::Ok(OkResponse::WithValue(Value::Number(0.0))));
    }

    #[test]
    fn should_parse_ok_response_with_list() {
        assert_eq!(Response::parse_ttp("+OK \"list\":[\"AecInput1\" \"AudioMeter2\" \"AudioMeter4\" \"DEVICE\" \"DanteInput1\" \"DanteOutput1\" \"Level1\" \"Level2\" \"Level3\" \"Mixer1\" \"NoiseGenerator1\" \"Output1\" \"Router1\" \"ToneGenerator1\" \"ToneGenerator2\" \"USBInput1\" \"USBOutput1\"]").unwrap(), Response::Ok(OkResponse::WithList(vec![
                Value::String("AecInput1".to_owned()),
                Value::String("AudioMeter2".to_owned()),
                Value::String("AudioMeter4".to_owned()),
                Value::String("DEVICE".to_owned()),
                Value::String("DanteInput1".to_owned()),
                Value::String("DanteOutput1".to_owned()),
                Value::String("Level1".to_owned()),
                Value::String("Level2".to_owned()),
                Value::String("Level3".to_owned()),
                Value::String("Mixer1".to_owned()),
                Value::String("NoiseGenerator1".to_owned()),
                Value::String("Output1".to_owned()),
                Value::String("Router1".to_owned()),
                Value::String("ToneGenerator1".to_owned()),
                Value::String("ToneGenerator2".to_owned()),
                Value::String("USBInput1".to_owned()),
                Value::String("USBOutput1".to_owned())
            ])));
    }

    #[test]
    fn should_parse_publish_token() {
        assert_eq!(Response::parse_ttp("! \"publishToken\":\"MyLevel4CH1\" \"value\":6.000000").unwrap(), Response::PublishToken(PublishToken {
            label: "MyLevel4CH1".to_owned(),
            value: vec![Value::Number(6.0)]
        }));
        assert_eq!(Response::parse_ttp("! \"publishToken\":\"MyLevel4ALL\" \"value\":[5.200000 3.000000 -10.000000 -60.000000]").unwrap(), Response::PublishToken(PublishToken {
            label: "MyLevel4ALL".to_owned(),
            value: vec![
                Value::Number(5.2),
                Value::Number(3.0),
                Value::Number(-10.0),
                Value::Number(-60.0)
            ]
        }));
    }

    #[test]
    fn should_parse_err() {
        assert_eq!(Response::parse_ttp("-ERR address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}").unwrap(), Response::Err(ErrResponse {
            message: "address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}".to_owned()
        }));
        assert_eq!(Response::parse_ttp("-ERR address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}\nAAAAA").unwrap(), Response::Err(ErrResponse {
            message: "address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}".to_owned()
        }));
        assert_eq!(Response::parse_ttp("-ERR").unwrap(), Response::Err(ErrResponse {
            message: "".to_owned()
        }));
    }
}