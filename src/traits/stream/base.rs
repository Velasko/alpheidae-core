use std::sync::Weak;

pub trait Stream {
    fn write(&mut self, data: Vec<u8>) -> anyhow::Result<()>;
}

pub trait Pipe: 'static + Send + Sync {
    fn get_id(&self) -> usize;
    fn get_output_channels(&self) -> &Vec<Weak<dyn Stream>>;
    fn read(&self) -> anyhow::Result<Option<Vec<u8>>>;

    fn add_output_channel(&self, channel: Weak<dyn Stream>);
    fn create_channel(&self) -> Weak<dyn Stream>;
}
