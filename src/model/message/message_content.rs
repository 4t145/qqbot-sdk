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

impl std::fmt::Display for MessageSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageSegment::Text(s) => write!(f, "{}", encode(s.as_str())),
            MessageSegment::At(id) => write!(f, "<@!{}>", id),
            MessageSegment::AtAll => write!(f, "@everyone"),
            MessageSegment::Channel(id) => write!(f, "<#{}>", id),
            MessageSegment::Emoji(id) => write!(f, "<emoji:{}>", id),
        }
    }
}

impl FromStr for MessageSegment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // don't parse channel, because it's not well supported
        if s == "<@!everyone>" {
            Ok(MessageSegment::AtAll)
        } else if s.starts_with("<@!") && s.ends_with('>') {
            let id = s[3..s.len() - 1].parse::<u64>().map_err(|_| "Invalid id")?;
            Ok(MessageSegment::At(id))
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
impl std::fmt::Display for MessageContent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for seg in &self.segments {
            write!(f, "{}", seg)?;
        }
        Ok(())
    }
}

impl FromStr for MessageContent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.replace("@everyone", "<@!everyone>");
        let mut content = MessageContent::new();
        let mut buf = String::new();
        let mut seg_start = Option::<usize>::None;
        for (idx, c) in s.chars().enumerate() {
            match c {
                '<' => {
                    if seg_start.is_some() {
                        return Err(format!("Invalid '<' at {}", idx));
                    }
                    if !buf.is_empty() {
                        content.segments.push(buf.parse()?);
                    }
                    buf = String::new();
                    buf.push(c);
                    seg_start.replace(idx);
                    continue;
                }
                '>' => {
                    let Some(_lb_idx) = seg_start.take() else {
                        return Err(format!("Invalid '>' at {}", idx));
                    };
                    buf.push(c);
                    content.segments.push(buf.parse()?);
                    buf = String::new();
                }
                _ => {
                    buf.push(c);
                }
            }
        }
        if !buf.is_empty() {
            content.segments.push(buf.parse()?);
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
