use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum Component {
    Literal(String),
    Array(Vec<Component>),
    Object {
        #[serde(skip_serializing_if = "Option::is_none")]
        bold: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        italic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        underlined: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        strikethrough: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        obfuscated: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        color: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        insertion: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        font: Option<String>,
        #[serde(flatten)]
        contents: ComponentContents,
        #[serde(rename = "extra", default, skip_serializing_if = "Vec::is_empty")]
        siblings: Vec<Component>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum ComponentContents {
    Literal {
        text: String,
    },
    Translatable {
        #[serde(rename = "translate")]
        key: String,
        #[serde(rename = "with", default, skip_serializing_if = "Vec::is_empty")]
        args: Vec<Component>,
    },
    Selector {
        #[serde(rename = "selector")]
        pattern: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        separator: Option<Box<Component>>,
    },
    Keybind {
        #[serde(rename = "keybind")]
        name: String,
    },
    Nbt {
        #[serde(rename = "nbt")]
        nbt_path_pattern: String,
        #[serde(rename = "interpret", default)]
        interpreting: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        separator: Option<Box<Component>>,
    },
}
