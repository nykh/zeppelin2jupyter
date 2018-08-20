extern crate serde_json;
extern crate failure;

use self::serde_json::Value;
use std::io::{Read, Write, BufReader, BufWriter};
use std::fs::File;

use self::failure::Error;

type JupyterJson = Value;
type ZeppelinJson = Value;
type JupyterCell = Value;
type ZeppelinCell = Value;
type ValArray = Value;
type ValString = Value;


fn read_file(path: &str) -> Result<String, Error> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents)?;
    Ok(contents)
}

fn write_file(path: &str, content: &String) -> Result<(), Error> {
    let file = File::create(path)?;
    let mut buf_writer = BufWriter::new(file);
    buf_writer.write(content.as_bytes())?;
    Ok(())
}

fn optionally_insert_title_node(z: &ZeppelinCell) -> Vec<JupyterCell> {
    match z["title"].as_str() {
        Some(title) => {
            let md_title = "### ".to_string() + title;
            vec!(json!({
                "cell_type": "markdown",
                "metadata": {},
                "source": [Value::String(md_title)]
                }))
        },
        None => Vec::new(),
    }
}

fn multiline_string_to_lines(s: &ValString) -> ValArray {
    let lines: Vec<&str> = s.as_str().unwrap().rsplitn(2, '\n').collect();
    let mut s2 = lines.get(1)
                      .map_or(Vec::new(), 
                              |s| s.lines().map(|line| {
                                let mut line_s = line.to_string();
                                line_s.push('\n');
                                line_s
                              }).collect());
    if let Some(last_line) = lines.get(0) {
        s2.push(last_line.to_string()); 
    }
    Value::Array(s2.into_iter().map(|s| Value::String(s)).collect())
}

const IMAGE_PREFIX: &'static str = "<div style='width:auto;height:auto'><img src=data:image/png;base64,";
const IMAGE_POSTFIX: &'static str = " style='width=auto;height:auto'><div>\n";

fn convert_codecell_outputs(zouts: &ValArray) -> ValArray {
    let mut jouts = Vec::new();
    for out in zouts.as_array().unwrap_or(&Vec::new()) {
        let o = out.as_object().unwrap();
        match o["type"].as_str() {
            Some("TEXT") => {
                let text = multiline_string_to_lines(&o["data"]);
                jouts.push(json!({
                    "name": "stdout",
                    "output_type": "stream",
                    "text": text
                }))
            },
            Some("HTML") => {
                // Markdown is ruled out
                let data = o["data"].as_str().unwrap();
                if &data[..IMAGE_PREFIX.len()] == IMAGE_PREFIX {
                    let mut image: String = data.chars().skip(IMAGE_PREFIX.len()).collect();
                    let newlen = image.len() - IMAGE_POSTFIX.len();
                    image.truncate(newlen);
                    jouts.push(json!({
                        "data": {
                            "image/png": Value::String(image),
                            "text/plain": []
                        },
                        "metadata": {},
                        "output_type": "display_data"
                    }))
                }
            }
            Some(&_) | None => (),
        }
    }
    Value::Array(jouts)
}

/// Convert a single cell
fn convert_cell(z: &ZeppelinCell) -> Vec<JupyterCell> {
    match z["config"]["editorSetting"]["language"].as_str() {
        Some("markdown") => {
            let mut source = multiline_string_to_lines(&z["text"]);
            source.as_array_mut().unwrap().remove(0);  // skip the "%md" line
            vec!(json!({
                "cell_type": "markdown",
                "metadata": {},
                "source": source
            }))
        },
        Some(&_) | None => {
            let mut output = optionally_insert_title_node(z);
            let source = multiline_string_to_lines(&z["text"]);
            let outputs = convert_codecell_outputs(&z["results"]["msg"]);

            output.push(json!({
            "cell_type": "code",
            "execution_count": null,
            "metadata": {},
            "outputs": outputs,
            "source": source
            }));

            output
        }
    }
}

/// Converts a zeppelin json to a jupyter json
fn convert_json(z: &ZeppelinJson) -> JupyterJson {
    let zcells = z["paragraphs"].as_array().unwrap();
    let jcells = Value::Array(zcells.into_iter()
                             .flat_map(convert_cell)
                             .collect::<Vec<_>>());
    json!({
        "cells": jcells,
        "metadata": {
            "kernelspec": {
                "display_name": "Scala",
                "language": "scala",
                "name": "scala"
            },
            "language_info": {
                "file_extension": ".scala",
                "name": "scala",
                "nbconvert_exporter": "scala",
                "pygments_lexer": "scala",
                "version": null
            }
        },
        "nbformat": 4,
        "nbformat_minor": 2
    })
}

/// Converts a zeppelin file to a jupyter file
pub fn convert(src: &str, dst: &str) -> Result<(), Error> {
    let s = read_file(src)?;
    let z = serde_json::from_str(&s)?;
    let j = serde_json::to_string_pretty(&convert_json(&z))?;
    write_file(dst, &j)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn multiline_string_to_lines_works_on_null_case() {
        let lines = multiline_string_to_lines(&json!(""));
        let lines_vec = lines.as_array().unwrap();
        assert_eq!(lines_vec.len(), 1);
        assert_eq!(lines_vec[0], "");
    }

    #[test]
    fn multiline_string_to_lines_works_on_simple_case() {
        let lines = multiline_string_to_lines(&json!("Hello\nWorld\nMan"));
        let lines_vec = lines.as_array().unwrap();
        assert_eq!(lines_vec.len(), 3);
        assert_eq!(lines_vec[0], "Hello\n");
        assert_eq!(lines_vec[1], "World\n");
        assert_eq!(lines_vec[2], "Man");
    }

    #[test]
    fn convert_outputs_works_on_null_case() {
        let outs = convert_codecell_outputs(&json!([]));
        let outs_vec = outs.as_array().unwrap();
        assert_eq!(outs_vec.len(), 0);
    }

    #[test]
    fn convert_outputs_works_on_simple_case() {
        let outs = convert_codecell_outputs(&json!([{
                                    "type": "TEXT",
                                    "data": "simple\nmulti-line\nstring"
                                    }]));
        let outs_vec = outs.as_array().unwrap();
        assert_eq!(outs_vec.len(), 1);
        assert_eq!(outs_vec[0]["name"].as_str(), Some("stdout"));
        assert_eq!(outs_vec[0]["output_type"].as_str(), Some("stream"));
        assert_eq!(outs_vec[0]["text"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn can_convert_markdown_cell() {
        let md_cell = json!({
            "text": "%md\n# Hello\nThis is an example **markdown**",
            "user": "anonymous",
            "config": {
                "editorSetting": {
                    "language": "markdown",
                },
            },
            "settings": {
                "params": {},
                "forms": {}
            },
            "results": {
                "code": "SUCCESS",
                "msg": [{
                    "type": "HTML",
                    "data": "doesn't matter"
                }]
            }
        });

        assert_eq!(convert_cell(&md_cell)[0], json!({
                "cell_type": "markdown",
                "metadata": {},
                "source": ["# Hello\n", "This is an example **markdown**"]
            }));
    }
}