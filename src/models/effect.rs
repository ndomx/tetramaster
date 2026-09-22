#[derive(PartialEq, Debug)]
pub enum Effect {
    Attack,
    DirectCapture,
}

impl Effect {
    pub fn priority(&self) -> u8 {
        match self {
            Effect::Attack => 0,
            Effect::DirectCapture => 1,
        }
    }
}
