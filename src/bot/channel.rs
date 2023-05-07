pub struct Channel {
    pub channel_id: u64,
}

impl Channel {
    pub fn new(channel_id: u64) -> Self {
        Self { channel_id }
    }
}
