use crate::commands::{RawCommandArgs, WholeStreamCommand};
use crate::errors::ShellError;
use crate::prelude::*;

pub struct Autoview;

#[derive(Deserialize)]
pub struct AutoviewArgs {}

impl WholeStreamCommand for Autoview {
    fn name(&self) -> &str {
        "autoview"
    }

    fn signature(&self) -> Signature {
        Signature::build("autoview")
    }

    fn usage(&self) -> &str {
        "View the contents of the pipeline as a table or list."
    }

    fn run(
        &self,
        args: CommandArgs,
        registry: &CommandRegistry,
    ) -> Result<OutputStream, ShellError> {
        Ok(args.process_raw(registry, autoview)?.run())
    }
}

pub fn autoview(
    AutoviewArgs {}: AutoviewArgs,
    mut context: RunnableContext,
    raw: RawCommandArgs,
) -> Result<OutputStream, ShellError> {
    Ok(OutputStream::new(async_stream_block! {
        let input = context.input.drain_vec().await;

        if input.len() > 0 {
            if let Tagged {
                item: Value::Primitive(Primitive::Binary(_)),
                ..
            } = input[0usize]
            {
                let binary = context.get_command("binaryview");
                if let Some(binary) = binary {
                    let result = binary.run(raw.with_input(input), &context.commands, false);
                    result.collect::<Vec<_>>().await;
                } else {
                    for i in input {
                        match i.item {
                            Value::Primitive(Primitive::Binary(b)) => {
                                use pretty_hex::*;
                                println!("{:?}", b.hex_dump());
                            }
                            _ => {}
                        }
                    }
                };
            } else if is_single_origined_text_value(&input) {
                let text = context.get_command("textview");
                if let Some(text) = text {
                    let result = text.run(raw.with_input(input), &context.commands, false);
                    result.collect::<Vec<_>>().await;
                } else {
                    for i in input {
                        match i.item {
                            Value::Primitive(Primitive::String(s)) => {
                                println!("{}", s);
                            }
                            _ => {}
                        }
                    }
                }
            } else if is_single_text_value(&input) {
                for i in input {
                    match i.item {
                        Value::Primitive(Primitive::String(s)) => {
                            println!("{}", s);
                        }
                        _ => {}
                    }
                }
            } else {
                let table = context.expect_command("table");
                let result = table.run(raw.with_input(input), &context.commands, false);
                result.collect::<Vec<_>>().await;
            }
        }
    }))
}

fn is_single_text_value(input: &Vec<Tagged<Value>>) -> bool {
    if input.len() != 1 {
        return false;
    }
    if let Tagged {
        item: Value::Primitive(Primitive::String(_)),
        ..
    } = input[0]
    {
        true
    } else {
        false
    }
}

fn is_single_origined_text_value(input: &Vec<Tagged<Value>>) -> bool {
    if input.len() != 1 {
        return false;
    }
    if let Tagged {
        item: Value::Primitive(Primitive::String(_)),
        tag: Tag {
            origin: Some(origin),
            ..
        },
    } = input[0]
    {
        origin != uuid::Uuid::nil()
    } else {
        false
    }
}
