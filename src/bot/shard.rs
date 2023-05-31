#[derive(Debug, Clone)]
pub struct Shards {
    pub total: u32,
    pub using_shards: Vec<u32>,
}

impl Shards {
    pub fn new(total: u32, shards: impl IntoIterator<Item = u32>) -> Self {
        let mut using_shards = shards
            .into_iter()
            .filter(|idx| *idx < total)
            .collect::<Vec<_>>();
        using_shards.dedup();
        Self {
            total,
            using_shards,
        }
    }

    pub fn new_all(total: u32) -> Self {
        Self {
            total,
            using_shards: (0..total).collect(),
        }
    }

    pub fn new_standalone() -> Self {
        Self {
            total: 1,
            using_shards: vec![0],
        }
    }
}
