use std::fmt::Display;

#[derive(PartialEq, Eq)]
pub enum BattleClass {
    Physical,
    Magic,
    Flexible,
    Assault,
}

impl Display for BattleClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chr = match self {
            BattleClass::Physical => 'P',
            BattleClass::Magic => 'M',
            BattleClass::Flexible => 'X',
            BattleClass::Assault => 'A',
        };

        write!(f, "{}", chr)
    }
}
