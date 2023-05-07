use std::str::FromStr;

fn encode<'a>(s: impl Into<&'a str>) -> String {
    s.into()
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn decode<'a>(s: impl Into<&'a str>) -> String {
    s.into()
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageSegment {
    Text(String),
    At(u64),
    AtAll,
    Channel(u64),
    Emoji(u64),
}

impl ToString for MessageSegment {
    fn to_string(&self) -> String {
        match self {
            MessageSegment::Text(s) => encode(s.as_str()),
            MessageSegment::At(id) => format!("<@!{}>", id),
            MessageSegment::AtAll => "@everyone".to_string(),
            MessageSegment::Channel(id) => format!("<#{}>", id),
            MessageSegment::Emoji(id) => format!("<emoji:{}>", id),
        }
    }
}

impl FromStr for MessageSegment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("<@!") && s.ends_with('>') {
            let id = s[3..s.len() - 1].parse::<u64>().map_err(|_| "Invalid id")?;
            Ok(MessageSegment::At(id))
        } else if s == "@everyone" {
            Ok(MessageSegment::AtAll)
        } else if s.starts_with("<#") && s.ends_with('>') {
            let id = s[2..s.len() - 1].parse::<u64>().map_err(|_| "Invalid id")?;
            Ok(MessageSegment::Channel(id))
        } else if s.starts_with("<emoji:") && s.ends_with('>') {
            let id = s[7..s.len() - 1].parse::<u64>().map_err(|_| "Invalid id")?;
            Ok(MessageSegment::Emoji(id))
        } else {
            Ok(MessageSegment::Text(decode(s)))
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct MessageContent {
    pub segments: Vec<MessageSegment>,
}

impl ToString for MessageContent {
    fn to_string(&self) -> String {
        self.segments
            .iter()
            .map(|seg| seg.to_string())
            .collect::<Vec<_>>()
            .join("")
    }
}

impl FromStr for MessageContent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut content = MessageContent::new();
        let mut buf = String::new();
        let mut seg_start = Option::<usize>::None;
        for (idx, c) in s.chars().enumerate() {
            match c {
                '<' => {
                    content.segments.push(MessageSegment::Text(buf));
                    buf = String::new();
                    if seg_start.is_some() {
                        return Err(format!("Invalid '<' at {}", idx));
                    }
                    seg_start.replace(idx);
                }
                '>' => {
                    if let Some(lb_idx) = seg_start.take() {
                        content.segments.push(s[lb_idx..=idx].parse()?);
                    } else {
                        return Err(format!("Invalid '>' at {}", idx));
                    }
                }
                '#' => {
                    seg_start.replace(idx);
                }
                _ => {
                    if seg_start.is_none() {
                        buf.push(c);
                    }
                }
            }
        }
        if !buf.is_empty() {
            content.segments.push(MessageSegment::Text(buf));
        }
        Ok(content)
    }
}
impl MessageContent {
    pub fn new() -> Self {
        MessageContent {
            segments: Vec::new(),
        }
    }

    pub fn text(&mut self, text: impl Into<String>) -> &mut Self {
        self.segments.push(MessageSegment::Text(text.into()));
        self
    }

    pub fn at(&mut self, id: u64) -> &mut Self {
        self.segments.push(MessageSegment::At(id));
        self
    }

    pub fn at_all(&mut self) -> &mut Self {
        self.segments.push(MessageSegment::AtAll);
        self
    }

    pub fn link_channel(&mut self, id: u64) -> &mut Self {
        self.segments.push(MessageSegment::Channel(id));
        self
    }

    pub fn emoji(&mut self, id: u64) -> &mut Self {
        self.segments.push(MessageSegment::Emoji(id));
        self
    }
}
