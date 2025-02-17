use crate::data::{Dictionary, Value};
use crate::errors::ShellError;
use crate::prelude::*;
use crate::TaggedDictBuilder;

use crate::commands::WholeStreamCommand;
use crate::parser::registry::Signature;
use indexmap::IndexMap;

pub struct Env;

impl WholeStreamCommand for Env {
    fn name(&self) -> &str {
        "env"
    }

    fn signature(&self) -> Signature {
        Signature::build("env")
    }

    fn usage(&self) -> &str {
        "Get the current environment."
    }

    fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        env(args, registry)
    }
}

pub fn get_environment(tag: Tag) -> Result<Tagged<Value>, Box<dyn std::error::Error>> {
    let mut indexmap = IndexMap::new();

    let path = std::env::current_dir()?;
    indexmap.insert("cwd".to_string(), Value::path(path).tagged(tag));

    if let Some(home) = dirs::home_dir() {
        indexmap.insert("home".to_string(), Value::path(home).tagged(tag));
    }

    let temp = std::env::temp_dir();
    indexmap.insert("temp".to_string(), Value::path(temp).tagged(tag));

    let mut dict = TaggedDictBuilder::new(tag);
    for v in std::env::vars() {
        dict.insert(v.0, Value::string(v.1));
    }
    if !dict.is_empty() {
        indexmap.insert("vars".to_string(), dict.into_tagged_value());
    }

    Ok(Value::Row(Dictionary::from(indexmap)).tagged(tag))
}

pub fn env(args: CommandArgs, registry: &CommandRegistry) -> Result<OutputStream, ShellError> {
    let args = args.evaluate_once(registry)?;

    let mut env_out = VecDeque::new();
    let tag = args.call_info.name_tag;

    let value = get_environment(tag)?;
    env_out.push_back(value);

    Ok(env_out.to_output_stream())
}
