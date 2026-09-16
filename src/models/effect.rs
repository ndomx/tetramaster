#[derive(PartialEq, Debug)]
pub enum Effect {
    Attack,
    Capture,
}

impl Effect {
    pub fn priority(&self) -> u8 {
        match self {
            Effect::Attack => 0,
            Effect::Capture => 1,
        }
    }
}
