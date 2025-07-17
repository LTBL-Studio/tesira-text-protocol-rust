//! Implementation of protcol commands and basic blocs

pub mod commands;
pub mod parser;

use chrono::{Datelike, naive::NaiveDateTime};
use parser::parse_response;
use std::{collections::HashMap, fmt::Display, time::Duration};
use thiserror::Error;

use crate::builder::CommandBuilder;

/// Name of block a command can operate on
pub type InstanceTag = String;

/// Value of an index
pub type IndexValue = u64;

/// A client command that can be sent to device
#[derive(Debug, Clone)]
pub struct Command<'a> {
    /// Block instance name to apply command on
    pub instance_tag: InstanceTag,
    /// Command string to trigger
    ///
    /// See [commands] module for predefined command strings
    pub command: &'a str,
    /// Attribute to apply command on
    pub attribute: &'a str,
    /// Optional indexes to specify command target
    pub indexes: Vec<IndexValue>,
    /// Optional values to add at command end
    pub values: Vec<String>,
}

/// Conversion trait to Tesira Text Protocol
pub trait IntoTTP {
    /// Convert this type to Tesira Text Protocol value
    fn into_ttp(self) -> String;
}

impl<'a> Command<'a> {
    /// Get a builder to construct valid commands
    pub fn builder() -> CommandBuilder {
        CommandBuilder
    }

    /// Create a new "get" command
    pub fn new_get(
        instance_tag: impl Into<String>,
        attribute: &'a str,
        indexes: impl Into<Vec<IndexValue>>,
    ) -> Self {
        Command {
            instance_tag: instance_tag.into(),
            command: commands::COMMAND_GET,
            attribute,
            indexes: indexes.into(),
            values: Vec::new(),
        }
    }

    /// Create a new "set" command
    pub fn new_set(
        instance_tag: impl Into<String>,
        attribute: &'a str,
        indexes: impl Into<Vec<IndexValue>>,
        value: impl IntoTTP,
    ) -> Self {
        Command {
            instance_tag: instance_tag.into(),
            command: commands::COMMAND_SET,
            attribute,
            indexes: indexes.into(),
            values: vec![value.into_ttp()],
        }
    }

    /// Create a new "increment" command
    pub fn new_increment(
        instance_tag: impl Into<String>,
        attribute: &'a str,
        indexes: impl Into<Vec<IndexValue>>,
        amount: impl IntoTTP,
    ) -> Self {
        Command {
            instance_tag: instance_tag.into(),
            command: commands::COMMAND_INCREMENT,
            attribute,
            indexes: indexes.into(),
            values: vec![amount.into_ttp()],
        }
    }

    /// Create a new "decrement" command
    pub fn new_decrement(
        instance_tag: impl Into<String>,
        attribute: &'a str,
        indexes: impl Into<Vec<IndexValue>>,
        amount: impl IntoTTP,
    ) -> Self {
        Command {
            instance_tag: instance_tag.into(),
            command: commands::COMMAND_DECREMENT,
            attribute,
            indexes: indexes.into(),
            values: vec![amount.into_ttp()],
        }
    }

    /// Create a new "subscribe" command
    pub fn new_subscribe(
        instance_tag: impl Into<String>,
        attribute: &'a str,
        indexes: impl Into<Vec<IndexValue>>,
        identifier: impl Into<String>,
    ) -> Self {
        Command {
            instance_tag: instance_tag.into(),
            command: commands::COMMAND_SUBSCRIBE,
            attribute,
            indexes: indexes.into(),
            values: vec![identifier.into().into_ttp()],
        }
    }

    /// Create a new "subscribe" command with a minimum rate
    pub fn new_subscribe_with_rate(
        instance_tag: impl Into<String>,
        attribute: &'a str,
        indexes: impl Into<Vec<IndexValue>>,
        identifier: impl Into<String>,
        rate: Duration,
    ) -> Self {
        Command {
            instance_tag: instance_tag.into(),
            command: commands::COMMAND_SUBSCRIBE,
            attribute,
            indexes: indexes.into(),
            values: vec![identifier.into().into_ttp(), rate.as_millis().into_ttp()],
        }
    }

    /// Create a new "unsubscribe" command
    pub fn new_unsubscribe(
        instance_tag: impl Into<String>,
        attribute: &'a str,
        indexes: impl Into<Vec<IndexValue>>,
        identifier: impl Into<String>,
    ) -> Self {
        Command {
            instance_tag: instance_tag.into(),
            command: commands::COMMAND_UNSUBSCRIBE,
            attribute,
            indexes: indexes.into(),
            values: vec![identifier.into().into_ttp()],
        }
    }
}

impl<'a> IntoTTP for Command<'a> {
    fn into_ttp(self) -> String {
        let mut cmd_ttp = format!("{} {} {}", self.instance_tag, self.command, self.attribute); // [instance tag] [command str] [attribute str]

        if !self.indexes.is_empty() {
            cmd_ttp.push(' ');
            cmd_ttp.push_str(
                self.indexes
                    .into_iter()
                    .map(|it| it.to_string())
                    .collect::<Vec<_>>()
                    .join(" ")
                    .as_str(),
            ); // [indexes...]
        }

        if !self.values.is_empty() {
            cmd_ttp.push(' ');
            cmd_ttp.push_str(self.values.join(" ").as_str()); // [values...]
        }

        cmd_ttp
    }
}

impl IntoTTP for String {
    fn into_ttp(self) -> String {
        self
    }
}

impl IntoTTP for bool {
    fn into_ttp(self) -> String {
        match self {
            true => "true".to_owned(),
            false => "false".to_owned(),
        }
    }
}

impl IntoTTP for i32 {
    fn into_ttp(self) -> String {
        self.to_string()
    }
}

impl IntoTTP for u128 {
    fn into_ttp(self) -> String {
        self.to_string()
    }
}

impl IntoTTP for u64 {
    fn into_ttp(self) -> String {
        self.to_string()
    }
}

impl IntoTTP for f64 {
    fn into_ttp(self) -> String {
        self.to_string()
    }
}

impl IntoTTP for NaiveDateTime {
    fn into_ttp(self) -> String {
        format!(
            "\"{}:{}:{}\"",
            self.format("%H:%M:%S"),
            self.month(),
            self.format("%d:%Y")
        )
    }
}

/// A response from device to a command
#[derive(Debug, Clone, PartialEq)]
pub enum Response {
    /// Command was executed and returned a positive response
    Ok(OkResponse),
    /// An error occured during command execution
    Err(ErrResponse),
    /// A value update for a subscription
    PublishToken(PublishToken),
}

/// An error produced by device in response to a command
#[derive(Debug, Clone, PartialEq)]
pub struct ErrResponse {
    /// Device message decribing the error
    pub message: String,
}

impl Display for ErrResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// A positive response to a command
#[derive(Debug, Clone, PartialEq)]
pub enum OkResponse {
    /// Everything Ok, no more information
    Ok,
    /// A value was provided in return to command
    WithValue(Value),
    /// A list of values was provided in return to command
    WithList(Vec<Value>),
}

/// A value update of a subscription
#[derive(Debug, Clone, PartialEq)]
pub struct PublishToken {
    /// Subscription identifier
    pub label: String,
    /// Value updated
    pub value: Value,
}

/// A structured value from Tesira devices
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// A floating point number
    Number(f64),
    /// A boolean value
    Boolean(bool),
    /// Any string value
    String(String),
    /// A JSON-like object key-value map
    Map(HashMap<String, Value>),
    /// A sequence of heterogenous values
    Array(Vec<Value>),
    /// A constant value described by a string such as "DHCP", "LINK_1_GB", etc.
    Constant(String),
}

impl Response {
    /// Parse ttp string into response
    pub fn parse_ttp(source: &str) -> Result<Self, Error> {
        parse_response(source).map(|it| it.1).map_err(|e| match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => Error::ParseError(e),
            nom::Err::Incomplete(_e) => Error::UnexpectedEnd,
        })
    }
}

/// A parsing error of response
#[derive(Debug, Error)]
pub enum Error<'a> {
    /// Error while parsing response
    #[error("Response parse error: {0}")]
    ParseError(nom::error::Error<&'a str>),
    /// More data is required to complete response parsing
    #[error("Unexpected end of input")]
    UnexpectedEnd,
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::proto::ErrResponse;
    use crate::proto::OkResponse;
    use crate::proto::PublishToken;
    use crate::proto::Response;
    use crate::proto::Value;
    use chrono::NaiveDateTime;
    use pretty_assertions::assert_eq;

    use super::Command;
    use super::IntoTTP;

    #[test]
    fn should_serialize_date() {
        assert_eq!(
            NaiveDateTime::parse_from_str("2025-06-01T12:56:43.000Z", "%+")
                .unwrap()
                .into_ttp(),
            "\"12:56:43:6:01:2025\""
        )
    }

    #[test]
    fn should_serialize_get_alias_command() {
        assert_eq!(
            Command::new_get("SESSION", "aliases", []).into_ttp(),
            "SESSION get aliases"
        );
    }

    #[test]
    fn should_serialize_get_command() {
        assert_eq!(
            Command::new_get("Level3", "level", [2]).into_ttp(),
            "Level3 get level 2"
        );
    }

    #[test]
    fn should_serialize_set_command() {
        assert_eq!(
            Command::new_set("level3", "mute", [3], true).into_ttp(),
            "level3 set mute 3 true"
        );

        assert_eq!(
            Command::new_set("level3", "mute", [0], true).into_ttp(),
            "level3 set mute 0 true"
        );
    }

    #[test]
    fn should_parse_simple_ok_response() {
        assert_eq!(
            Response::parse_ttp("+OK").unwrap(),
            Response::Ok(OkResponse::Ok)
        );
    }

    #[test]
    fn should_parse_ok_response_with_value() {
        assert_eq!(
            Response::parse_ttp("+OK \"value\":0.000000").unwrap(),
            Response::Ok(OkResponse::WithValue(Value::Number(0.0)))
        );
    }

    #[test]
    fn should_parse_ok_response_with_empty_string_value() {
        assert_eq!(
            Response::parse_ttp("+OK \"value\":\"\"").unwrap(),
            Response::Ok(OkResponse::WithValue(Value::String("".to_owned())))
        );
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
            (
                "hostname".to_owned(),
                Value::String("TesiraForte05953601".to_owned()),
            ),
            (
                "defaultGatewayStatus".to_owned(),
                Value::String("0.0.0.0".to_owned()),
            ),
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
        assert_eq!(
            Response::parse_ttp("+OK \"value\":LINK_1_GB").unwrap(),
            Response::Ok(OkResponse::WithValue(Value::Constant(
                "LINK_1_GB".to_owned()
            )))
        );
    }

    #[test]
    fn should_parse_ok_response_with_nested_value() {
        let expected_value = Value::Map(HashMap::from([
            ("schemaVersion".to_owned(), Value::Number(2.0)),
            (
                "hostname".to_owned(),
                Value::String("TesiraForte05953601".to_owned()),
            ),
            (
                "defaultGatewayStatus".to_owned(),
                Value::String("0.0.0.0".to_owned()),
            ),
            (
                "networkInterfaceStatusWithName".to_owned(),
                Value::Array(vec![Value::Map(HashMap::from([
                    (
                        "interfaceId".to_owned(),
                        Value::String("control".to_owned()),
                    ),
                    (
                        "networkInterfaceStatus".to_owned(),
                        Value::Map(HashMap::from([
                            (
                                "macAddress".to_owned(),
                                Value::String("78:45:01:3d:86:92".to_owned()),
                            ),
                            (
                                "linkStatus".to_owned(),
                                Value::Constant("LINK_1_GB".to_owned()),
                            ),
                            (
                                "addressSource".to_owned(),
                                Value::Constant("DHCP".to_owned()),
                            ),
                            ("ip".to_owned(), Value::String("10.0.151.235".to_owned())),
                            (
                                "netmask".to_owned(),
                                Value::String("255.255.252.0".to_owned()),
                            ),
                            (
                                "dhcpLeaseObtainedDate".to_owned(),
                                Value::String("Wed Jun 26 16:45:27 UTC 2024".to_owned()),
                            ),
                            (
                                "dhcpLeaseExpiresDate".to_owned(),
                                Value::String("Thu Jun 27 16:45:27 UTC 2024".to_owned()),
                            ),
                            ("gateway".to_owned(), Value::String("10.0.148.1".to_owned())),
                        ])),
                    ),
                ]))]),
            ),
            (
                "dnsStatus".to_owned(),
                Value::Map(HashMap::from([
                    (
                        "primaryDNSServer".to_owned(),
                        Value::String("10.0.148.1".to_owned()),
                    ),
                    (
                        "secondaryDNSServer".to_owned(),
                        Value::String("".to_owned()),
                    ),
                    ("domainName".to_owned(), Value::String("".to_owned())),
                ])),
            ),
            ("mDNSEnabled".to_owned(), Value::Boolean(true)),
            ("telnetDisabled".to_owned(), Value::Boolean(true)),
            ("sshDisabled".to_owned(), Value::Boolean(false)),
            (
                "networkPortMode".to_owned(),
                Value::Constant("PORT_MODE_SEPARATE".to_owned()),
            ),
            ("rstpEnabled".to_owned(), Value::Boolean(false)),
            ("httpsEnabled".to_owned(), Value::Boolean(false)),
            ("igmpEnabled".to_owned(), Value::Boolean(false)),
            (
                "switchPortMode".to_owned(),
                Value::Constant("SWITCH_PORT_MODE_CONTROL_AND_MEDIA".to_owned()),
            ),
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
        assert_eq!(
            Response::parse_ttp("! \"publishToken\":\"MyLevel4CH1\" \"value\":6.000000").unwrap(),
            Response::PublishToken(PublishToken {
                label: "MyLevel4CH1".to_owned(),
                value: Value::Number(6.0)
            })
        );
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
        assert_eq!(
            Response::parse_ttp(
                "-ERR address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}"
            )
            .unwrap(),
            Response::Err(ErrResponse {
                message: "address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}"
                    .to_owned()
            })
        );
        assert_eq!(
            Response::parse_ttp(
                "-ERR address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}\nAAAAA"
            )
            .unwrap(),
            Response::Err(ErrResponse {
                message: "address not found: {\"deviceId\":0 \"classCode\":0 \"instanceNum\":0}"
                    .to_owned()
            })
        );
        assert_eq!(
            Response::parse_ttp("-ERR").unwrap(),
            Response::Err(ErrResponse {
                message: "".to_owned()
            })
        );
    }
}
