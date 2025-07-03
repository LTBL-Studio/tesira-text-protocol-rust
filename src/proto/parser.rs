use nom::{branch::alt, bytes::complete::{is_not, tag, take_until, take_while1}, character::complete::space1, combinator::{opt, rest, value}, multi::separated_list0, sequence::{delimited, pair, preceded, terminated}, IResult, Parser};

use super::{ErrResponse, OkResponse, PublishToken, Response, Value};

fn float_str(input: &str) -> IResult<&str, f64> {
    pair(
        pair(opt(tag("-")), take_while1(|c: char| c.is_ascii_digit())),
        opt(preceded(tag("."), take_while1(|c: char| c.is_ascii_digit())))
    ).map(|it: ((Option<&str>, &str), Option<&str>)| {
        let mut whole: i64 = it.0.1.parse().unwrap();
        if it.0.0.is_some() {
            whole *= -1
        }

        let fractional = it.1.map(|it: &str| {
            let trimmed_value = it.trim_end_matches('0');
            if trimmed_value.is_empty() {
                return 0_f64;
            }
            let mut value: i64 = trimmed_value.parse().unwrap();
            if whole < 0 {
                value *= -1
            }
            value as f64 / (10_i64.pow(trimmed_value.len() as u32)) as f64
        }).unwrap_or(0_f64);

        whole as f64 + fractional
    }).parse(input)
}

fn ttp_value(input: &str) -> IResult<&str, Value> {
    alt((
        delimited(tag("\""), is_not("\""), tag("\"")).map(|it: &str| Value::String(it.to_owned())),
        value(Value::Boolean(true), tag("true")),
        value(Value::Boolean(false), tag("false")),
        float_str.map(Value::Number)
    )).parse(input)
}

fn ttp_list_of_values(input: &str) -> IResult<&str, Vec<Value>> {
    delimited(tag("["),
        separated_list0(space1, ttp_value),
        tag("]")
    ).parse(input)
}

fn field(name: &str) -> impl Parser<&str, Output = &str, Error = nom::error::Error<&str>> {
    terminated(
        delimited(tag("\""), tag(name), tag("\"")),
        tag(":")
    )
}

fn ok_response(input: &str) -> IResult<&str, OkResponse> {
    let (input, extra) = preceded(tag("+OK"), opt(alt((
        preceded(preceded(space1, field("value")), ttp_value).map(OkResponse::WithValue),
        preceded(preceded(space1, field("list")), ttp_list_of_values).map(OkResponse::WithList)
    )))).parse(input)?;

    Ok((input, extra.unwrap_or(OkResponse::Ok)))
}

fn err_response(input: &str) -> IResult<&str, ErrResponse> {
    let (input, message) = preceded(
        tag("-ERR"),
        opt(preceded(space1, alt((
            take_until("\n"),
            rest
        ))))
    ).parse(input)?;

    Ok((input, ErrResponse {
        message: message.unwrap_or("").to_owned()
    }))
}

fn publish_token_response(input: &str) -> IResult<&str, PublishToken> {
    let (input, (label, value)) = preceded(tag("! \"publishToken\":"), pair(
        delimited(tag("\""), is_not("\""), tag("\"")),
        preceded(space1, preceded(field("value"), alt((
            ttp_list_of_values,
            ttp_value.map(|v| vec![v])
        ))))
    )).parse(input)?;
    
    Ok((input, PublishToken {
        label: label.to_owned(),
        value
    }))
}

pub fn parse_response(input: &str) -> IResult<&str, Response> {
    alt((
        ok_response.map(Response::Ok),
        err_response.map(Response::Err),
        terminated(publish_token_response, tag(" +OK")).map(|it| Response::Ok(OkResponse::WithPublishToken(it))),
        publish_token_response.map(Response::PublishToken)
    )).parse(input)
}

mod test {
    #[allow(unused_imports)]
    use crate::proto::parser::float_str;

    #[test]
    fn should_parse_float(){
        assert_eq!(float_str("0"), Ok(("", 0.0_f64)));
        assert_eq!(float_str("-0"), Ok(("", 0.0_f64)));
        assert_eq!(float_str("-15"), Ok(("", -15.0_f64)));
        assert_eq!(float_str("0.00000000000"), Ok(("", 0.0_f64)));
        assert_eq!(float_str("5.2000000000"), Ok(("", 5.2_f64)));
        assert_eq!(float_str("12"), Ok(("", 12.0_f64)));
        assert_eq!(float_str("12.000"), Ok(("", 12.0_f64)));
    }

}