pub mod parser;
use std::{collections::HashMap, fmt::Display, time::Duration};
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
    pub instance_tag: InstanceTag,
    pub attribute: String,
    pub args: Vec<String>
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
    pub value: Value
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    Boolean(bool),
    String(String),
    Map(HashMap<String, Value>),
    Array(Vec<Value>),
    Constant(String)
}

pub type InstanceTag = String;

impl Response {
    pub fn parse_ttp(source: &str) -> Result<Self, Error> {
        parse_response(source)
            .map(|it| it.1)
            .map_err(|e| match e {
                nom::Err::Error(e) | nom::Err::Failure(e) => Error::ParseError(e),
                nom::Err::Incomplete(_e) => {
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

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
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
    fn should_parse_ok_response_with_empty_string_value() {
        assert_eq!(Response::parse_ttp("+OK \"value\":\"\"").unwrap(), Response::Ok(OkResponse::WithValue(Value::String("".to_owned()))));
    }

    #[test]
    fn should_parse_ok_response_with_array_value() {

        let expected_value = Value::Array(vec![
            Value::Number(2.0),
            Value::String("TesiraForte05953601".to_owned()),
            Value::String("0.0.0.0".to_owned()),
            Value::Boolean(true),
            Value::Boolean(true),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Boolean(false),
            Value::Boolean(false),
        ]);

        assert_eq!(Response::parse_ttp("+OK \"value\":[2 \"TesiraForte05953601\" \"0.0.0.0\" true true false false false false]").unwrap(), Response::Ok(OkResponse::WithValue(expected_value)));
    }

    #[test]
    fn should_parse_ok_response_with_map_value() {

        let expected_value = Value::Map(HashMap::from([
            ("schemaVersion".to_owned(), Value::Number(2.0)),
            ("hostname".to_owned(), Value::String("TesiraForte05953601".to_owned())),
            ("defaultGatewayStatus".to_owned(), Value::String("0.0.0.0".to_owned())),
            ("mDNSEnabled".to_owned(), Value::Boolean(true)),
            ("telnetDisabled".to_owned(), Value::Boolean(true)),
            ("sshDisabled".to_owned(), Value::Boolean(false)),
            ("rstpEnabled".to_owned(), Value::Boolean(false)),
            ("httpsEnabled".to_owned(), Value::Boolean(false)),
            ("igmpEnabled".to_owned(), Value::Boolean(false)),
        ]));

        assert_eq!(Response::parse_ttp("+OK \"value\":{\"schemaVersion\":2 \"hostname\":\"TesiraForte05953601\" \"defaultGatewayStatus\":\"0.0.0.0\" \"mDNSEnabled\":true \"telnetDisabled\":true \"sshDisabled\":false \"rstpEnabled\":false \"httpsEnabled\":false \"igmpEnabled\":false}").unwrap(), Response::Ok(OkResponse::WithValue(expected_value)));
    }

    #[test]
    fn should_parse_ok_response_with_constant_value() {
        assert_eq!(Response::parse_ttp("+OK \"value\":LINK_1_GB").unwrap(), Response::Ok(OkResponse::WithValue(Value::Constant("LINK_1_GB".to_owned()))));
    }

    #[test]
    fn should_parse_ok_response_with_nested_value() {

        let expected_value = Value::Map(HashMap::from([
            ("schemaVersion".to_owned(), Value::Number(2.0)),
            ("hostname".to_owned(), Value::String("TesiraForte05953601".to_owned())),
            ("defaultGatewayStatus".to_owned(), Value::String("0.0.0.0".to_owned())),
            ("networkInterfaceStatusWithName".to_owned(), Value::Array(vec![
                Value::Map(HashMap::from([
                    ("interfaceId".to_owned(), Value::String("control".to_owned())),
                    ("networkInterfaceStatus".to_owned(), Value::Map(HashMap::from([
                        ("macAddress".to_owned(), Value::String("78:45:01:3d:86:92".to_owned())),
                        ("linkStatus".to_owned(), Value::Constant("LINK_1_GB".to_owned())),
                        ("addressSource".to_owned(), Value::Constant("DHCP".to_owned())),
                        ("ip".to_owned(), Value::String("10.0.151.235".to_owned())),
                        ("netmask".to_owned(), Value::String("255.255.252.0".to_owned())),
                        ("dhcpLeaseObtainedDate".to_owned(), Value::String("Wed Jun 26 16:45:27 UTC 2024".to_owned())),
                        ("dhcpLeaseExpiresDate".to_owned(), Value::String("Thu Jun 27 16:45:27 UTC 2024".to_owned())),
                        ("gateway".to_owned(), Value::String("10.0.148.1".to_owned())),
                    ])))
                ]))
            ])),
            ("dnsStatus".to_owned(), Value::Map(HashMap::from([
                ("primaryDNSServer".to_owned(), Value::String("10.0.148.1".to_owned())),
                ("secondaryDNSServer".to_owned(), Value::String("".to_owned())),
                ("domainName".to_owned(), Value::String("".to_owned())),
            ]))),
            ("mDNSEnabled".to_owned(), Value::Boolean(true)),
            ("telnetDisabled".to_owned(), Value::Boolean(true)),
            ("sshDisabled".to_owned(), Value::Boolean(false)),
            ("networkPortMode".to_owned(), Value::Constant("PORT_MODE_SEPARATE".to_owned())),
            ("rstpEnabled".to_owned(), Value::Boolean(false)),
            ("httpsEnabled".to_owned(), Value::Boolean(false)),
            ("igmpEnabled".to_owned(), Value::Boolean(false)),
            ("switchPortMode".to_owned(), Value::Constant("SWITCH_PORT_MODE_CONTROL_AND_MEDIA".to_owned())),
        ]));

        assert_eq!(Response::parse_ttp("+OK \"value\":{\"schemaVersion\":2 \"hostname\":\"TesiraForte05953601\" \"defaultGatewayStatus\":\"0.0.0.0\" \"networkInterfaceStatusWithName\":[{\"interfaceId\":\"control\" \"networkInterfaceStatus\":{\"macAddress\":\"78:45:01:3d:86:92\" \"linkStatus\":LINK_1_GB \"addressSource\":DHCP \"ip\":\"10.0.151.235\" \"netmask\":\"255.255.252.0\" \"dhcpLeaseObtainedDate\":\"Wed Jun 26 16:45:27 UTC 2024\" \"dhcpLeaseExpiresDate\":\"Thu Jun 27 16:45:27 UTC 2024\" \"gateway\":\"10.0.148.1\"}}] \"dnsStatus\":{\"primaryDNSServer\":\"10.0.148.1\" \"secondaryDNSServer\":\"\" \"domainName\":\"\"} \"mDNSEnabled\":true \"telnetDisabled\":true \"sshDisabled\":false \"networkPortMode\":PORT_MODE_SEPARATE \"rstpEnabled\":false \"httpsEnabled\":false \"igmpEnabled\":false \"switchPortMode\":SWITCH_PORT_MODE_CONTROL_AND_MEDIA}").unwrap(), Response::Ok(OkResponse::WithValue(expected_value)));
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
            value: Value::Number(6.0)
        }));
        assert_eq!(Response::parse_ttp("! \"publishToken\":\"MyLevel4ALL\" \"value\":[5.200000 3.000000 -10.000000 -60.000000]").unwrap(), Response::PublishToken(PublishToken {
            label: "MyLevel4ALL".to_owned(),
            value: Value::Array(vec![
                Value::Number(5.2),
                Value::Number(3.0),
                Value::Number(-10.0),
                Value::Number(-60.0)
            ])
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