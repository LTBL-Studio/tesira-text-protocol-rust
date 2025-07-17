use codegen::{Enum, Function, Impl, Scope, Struct};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

type TesiraBlocks = HashMap<String, Block>;

#[derive(Debug, Deserialize)]
struct Block {
    group: String,
    attributes: Vec<BlockAttribute>,
}

#[derive(Debug, Deserialize)]
struct BlockAttribute {
    commands: Vec<AttributeCommand>,
    #[serde(rename = "commandstring", default)]
    name: String,
    description: String,
    #[serde(default)]
    indexes: Vec<AttributeIndex>,
    #[serde(flatten, default = "default_value_type")]
    value: AttributeValue,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "valuetype", rename_all = "lowercase")]
enum AttributeValue {
    None,
    Range {
        #[serde(rename = "valuemin")]
        min: Option<f64>,
        #[serde(rename = "valuemax")]
        max: Option<f64>,
    },
    Discrete {
        values: Vec<String>,
    },
    #[serde(rename = "cmdstr")]
    CommandAndString,
    Delay,
    Unbounded,
    #[serde(rename = "typeslope")]
    TypeSlope,
    #[serde(rename = "freqgain")]
    FreqencyAndGain,
    Date,
    #[serde(rename = "videoBandwidth")]
    VideoBandwidth,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AttributeCommand {
    Get,
    Set,
    Increment,
    Decrement,
    Toggle,
    Subscribe,
    Unsubscribe,
    #[serde(rename = "")]
    Empty,
    Dial,
    #[serde(rename = "speedDial")]
    SpeedDial,
    Redial,
    End,
    Flash,
    Send,
    Dtmf,
    Answer,
    Lconf,
    Resume,
    Hold,
    #[serde(rename = "offHook")]
    OffHook,
    #[serde(rename = "onHook")]
    OnHook,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum AttributeIndex {
    Channel,
    #[serde(rename = "AV channel")]
    AVChannel,
    #[serde(rename = "auxiliary audio channel")]
    AuxiliaryAudioChannel,
    Band,
    #[serde(alias = " filter")]
    Filter,
    Command,
    #[serde(rename = "input group")]
    InputGroup,
    #[serde(rename = "")]
    None,
    Line,
    #[serde(rename = "speed dial entry")]
    SpeedDialEntry,
    #[serde(rename = "call appearance")]
    CallAppearance,
    #[serde(rename = "call appearance index", alias = " call appearance index")]
    CallAppearanceIndex,
    Source,
    #[serde(alias = " output")]
    Output,
    Input,
    Room,
    Wall,
    Hostname,
    Port,
}

impl AttributeIndex {
    fn to_parameter_name(&self) -> &'static str {
        match self {
            AttributeIndex::Channel => "channel_index",
            AttributeIndex::AVChannel => "av_channel_index",
            AttributeIndex::AuxiliaryAudioChannel => "auxiliary_audio_channel_index",
            AttributeIndex::Band => "band",
            AttributeIndex::Filter => "filter",
            AttributeIndex::Command => "command",
            AttributeIndex::InputGroup => "input_group",
            AttributeIndex::None => {
                panic!("Attempt to create parameter name out of None attribute index")
            }
            AttributeIndex::Line => "line_index",
            AttributeIndex::SpeedDialEntry => "speed_dial_entry",
            AttributeIndex::CallAppearance => "call_appearance",
            AttributeIndex::CallAppearanceIndex => "call_appaearance_index",
            AttributeIndex::Source => "source_index",
            AttributeIndex::Output => "output_index",
            AttributeIndex::Input => "input_index",
            AttributeIndex::Room => "room_index",
            AttributeIndex::Wall => "wall_index",
            AttributeIndex::Hostname => "hostname",
            AttributeIndex::Port => "port",
        }
    }
}

fn to_fn_name(prefix: &str, value: &str) -> String {
    let mut final_value = value
        .trim()
        .chars()
        .map(|it| if it.is_whitespace() { '_' } else { it })
        .filter(|it| it.is_alphanumeric() || *it == '_')
        .flat_map(|it| it.to_lowercase())
        .collect::<String>();
    final_value = format!("{}{}", prefix, final_value);

    if final_value == "type" {
        final_value = format!("r#{}", final_value)
    }
    final_value
}

fn to_struct_name(value: &str, parent: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|it| it.is_alphanumeric() || it.is_whitespace())
        .flat_map(|it| it.to_lowercase())
        .fold(String::new(), |mut acc, it| {
            if acc.is_empty() {
                if it.is_numeric() {
                    acc.push_str(parent);
                }
                acc.push_str(&it.to_uppercase().collect::<String>());
            } else if acc[acc.len() - 1..acc.len()].trim().is_empty() {
                let replace = it.to_uppercase().collect::<String>();
                acc.replace_range(acc.len() - 1..acc.len(), &replace[0..1]);
                acc.push_str(&replace[1..]);
            } else {
                acc.push(it);
            }
            acc
        })
}

fn main() {
    let generated_dir = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("generated");
    fs::create_dir_all(&generated_dir).unwrap();

    let mut f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(generated_dir.join("tesira-blocks.rs"))
        .unwrap();

    let blocks: TesiraBlocks = serde_json::from_str(include_str!("tesira-blocks.json")).unwrap();

    let mut scope = Scope::new();
    let mut builder_impl = Impl::new("CommandBuilder");

    for (block_name, block) in blocks.into_iter() {
        let builder_type = format!("{}CommandBuilder", to_struct_name(&block_name, "Tesira"));

        let mut block_builder = Struct::new(&builder_type);
        block_builder
            .vis("pub")
            .doc(&format!(
                "Operate on block of type {}\n\nBlock type: {}\nBlock group: {}",
                block_name, block_name, block.group
            ));

        let instance_tag_var = if block_name == "Session Services" {
            "\"SESSION\""
        } else if block_name == "Device Services" {
            "\"DEVICE\""
        } else {
            block_builder.tuple_field("InstanceTag");
            "self.0"
        };

        let mut block_builder_impl = Impl::new(builder_type.clone());

        let builder_fn_name = if block_name == "Session Services" {
            "session".to_owned()
        } else if block_name == "Device Services" {
            "device".to_owned()
        } else {
            to_fn_name("", &block_name)
        };

        {
            let builder_fn = builder_impl
                .new_fn(&builder_fn_name)
                .arg_self()
                .doc(format!("Operate on block of type {}", block_name))
                .ret(builder_type.clone())
                .vis("pub");

            if block_name == "Session Services" || block_name == "Device Services" {
                builder_fn.line(format!("{}", builder_type));
            } else {
                builder_fn.arg("instance_tag", "impl Into<InstanceTag>")
                    .line(format!("{}(instance_tag.into())", builder_type));
            }
        }

        let mut discrete_types: HashMap<Vec<String>, String> = HashMap::new();
        {
            let mut bool_vec = vec!["false".to_owned(), "true".to_owned()];
            bool_vec.sort();
            discrete_types.insert(bool_vec, "bool".to_owned());
        }

        for attribute in block.attributes.iter() {
            for command in attribute.commands.iter() {
                let new_fn: Vec<(Function, Vec<(&'static str, String)>)> = match command {
                    AttributeCommand::Get => {
                        let extra_args: Vec<(&'static str, String)> = Vec::new();
                        let mut new_fn = Function::new(&to_fn_name("", &attribute.name));
                        new_fn
                            .vis("pub")
                            .ret("Command<'static>")
                            .doc(format!("Get {}", attribute.description))
                            .line("Command {")
                            .line("\tcommand: COMMAND_GET,")
                            .line("\tvalues: Vec::new(),");
                        vec![(new_fn, extra_args)]
                    }
                    AttributeCommand::Set => {
                        let mut extra_args: Vec<(&'static str, String)> = Vec::new();
                        let mut new_fn = Function::new(&to_fn_name("set_", &attribute.name));
                        new_fn
                            .vis("pub")
                            .ret("Command<'static>")
                            .doc(format!("Set {}", attribute.description))
                            .line("Command {")
                            .line("\tcommand: COMMAND_SET,");

                        let mut extra_fn = Vec::new();

                        match &attribute.value {
                            AttributeValue::None => {
                                new_fn.line("\tvalues: Vec::new(),");
                            }
                            AttributeValue::Discrete { values } => {
                                let mut sorted_values = values.clone();
                                sorted_values.sort();

                                let discrete_type =
                                    discrete_types.entry(sorted_values).or_insert_with(|| {
                                        let enum_name = format!(
                                            "{}",
                                            to_struct_name(
                                                &format!(
                                                    "{} {}",
                                                    block_name, &attribute.description
                                                ),
                                                "Tesira"
                                            )
                                        );

                                        let mut new_enum = Enum::new(enum_name.clone());
                                        new_enum.doc(&format!("Allowed values for {} on {}", attribute.description, block_name))
                                            .vis("pub")
                                            .allow("missing_docs");
                                        let mut new_enum_impl = Impl::new(enum_name.clone());
                                        new_enum_impl.impl_trait("IntoTTP");
                                        let convert_fn = new_enum_impl
                                            .new_fn("into_ttp")
                                            .ret("String")
                                            .arg_self()
                                            .line("match self {");

                                        for variant in values {
                                            let variant_name = to_struct_name(&variant, &enum_name);
                                            convert_fn.line(format!(
                                                "\tSelf::{} => \"{}\".to_owned(),",
                                                variant_name, variant
                                            ));
                                            new_enum.new_variant(variant_name);
                                        }

                                        convert_fn.line("}");

                                        scope.push_enum(new_enum);
                                        scope.push_impl(new_enum_impl);
                                        return enum_name;
                                    });

                                extra_args.push(("value", discrete_type.clone()));
                                new_fn.line("\tvalues: vec![value.into_ttp()],");

                                // TODO other descrete value
                            }
                            AttributeValue::Range {
                                min: _min,
                                max: _max,
                            } => {
                                extra_args.push(("value", "f64".to_owned()));
                                new_fn.line("\tvalues: vec![value.into_ttp()],");
                            }
                            AttributeValue::Unbounded => {
                                extra_args.push(("value", "impl IntoTTP".to_owned()));
                                new_fn.line("\tvalues: vec![value.into_ttp()],");
                            }
                            AttributeValue::Delay => {
                                extra_args.push(("value", "DelayValue".to_owned()));
                                new_fn.line("\tvalues: vec![value.into_ttp()],");
                            }
                            AttributeValue::TypeSlope => {
                                extra_args.push(("filter_type", "FilterType".to_owned()));
                                extra_args.push(("filter_slope", "FilterSlope".to_owned()));
                                new_fn.line("\tvalues: vec![format!(\"{{\\\"type\\\":{} \\\"slope\\\":{}}}\", filter_type.into_ttp(), filter_slope.into_ttp())],");
                            }
                            AttributeValue::FreqencyAndGain => {
                                extra_args.push(("freqency", "f64".to_owned()));
                                extra_args.push(("gain", "f64".to_owned()));
                                new_fn.line("\tvalues: vec![format!(\"{{\\\"frequency\\\":{} \\\"gain\\\":{}}}\", freqency.into_ttp(), gain.into_ttp())],");
                            }
                            AttributeValue::Date => {
                                extra_args.push(("value", "NaiveDateTime".to_owned()));
                                new_fn.line("\tvalues: vec![value.into_ttp()],");
                            }
                            AttributeValue::CommandAndString => continue, //TODO
                            AttributeValue::VideoBandwidth => continue, // Video Bandwidth not supported fo rnow
                        }

                        extra_fn.push((new_fn, extra_args));
                        extra_fn
                    }
                    AttributeCommand::Subscribe => {
                        let mut new_fn = Function::new(&to_fn_name("subscribe_", &attribute.name));
                        new_fn
                            .vis("pub")
                            .ret("Command<'static>")
                            .doc(format!("Subscribe to {} value update", attribute.description))
                            .line("Command {")
                            .line("\tcommand: COMMAND_SUBSCRIBE,")
                            .line("\tvalues: vec![subscription_label.into().into_ttp()],");

                        let mut new_fn_rate = Function::new(&format!("{}_with_rate", to_fn_name("subscribe_", &attribute.name)));
                        new_fn_rate
                            .vis("pub")
                            .ret("Command<'static>")
                            .doc(format!("Subscribe to {} value update", attribute.description))
                            .line("Command {")
                            .line("\tcommand: COMMAND_SUBSCRIBE,")
                            .line("\tvalues: vec![subscription_label.into().into_ttp(), min_rate.as_millis().into_ttp()],");

                        vec![
                            (new_fn, vec![("subscription_label", "impl Into<String>".to_owned())]),
                            (new_fn_rate, vec![
                                ("subscription_label", "impl Into<String>".to_owned()),
                                ("min_rate", "Duration".to_owned())
                            ])
                        ]
                    }
                    AttributeCommand::Unsubscribe => {
                        let mut new_fn = Function::new(&to_fn_name("unsubscribe_", &attribute.name));
                        new_fn
                            .vis("pub")
                            .ret("Command<'static>")
                            .doc(format!("Subscribe to {} value update", attribute.description))
                            .line("Command {")
                            .line("\tcommand: COMMAND_UNSUBSCRIBE,")
                            .line("\tvalues: vec![subscription_label.into().into_ttp()],");

                        vec![
                            (new_fn, vec![("subscription_label", "impl Into<String>".to_owned())])
                        ]
                    }
                    _ => continue, // TODO
                };

                for (mut new_fn, extra_args) in new_fn.into_iter() {
                    new_fn.line(format!("\tattribute: \"{}\",", attribute.name));
                    new_fn.arg_ref_self();
                    new_fn.line(format!("\tinstance_tag: {}.to_owned(),", instance_tag_var));

                    let mut indexes_param = Vec::new();
                    for index in attribute
                        .indexes
                        .iter()
                        .filter(|it| !matches!(it, AttributeIndex::None))
                    {
                        let param_name = index.to_parameter_name();
                        new_fn.arg(param_name, "IndexValue");
                        indexes_param.push(param_name);
                    }
                    new_fn.line(format!("\tindexes: vec![{}],", indexes_param.join(", ")));

                    new_fn.line("}");

                    for arg in extra_args {
                        new_fn.arg(arg.0, arg.1);
                    }

                    block_builder_impl.push_fn(new_fn);
                }
            }
        }

        scope.push_struct(block_builder);
        scope.push_impl(block_builder_impl);
    }

    scope.push_impl(builder_impl);

    f.write_all(scope.to_string().as_bytes()).unwrap();

    println!("cargo::rerun-if-changed=tesira-blocks.json");
    println!("cargo::rerun-if-changed=build.rs");
}
