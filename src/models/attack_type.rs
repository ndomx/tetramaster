use std::fmt::Display;

#[derive(PartialEq, Eq)]
pub enum AttackType {
    Physical,
    Magic,
    SelectDefense,
    SelectAll,
}

impl Display for AttackType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let chr = match self {
            AttackType::Physical => 'P',
            AttackType::Magic => 'M',
            AttackType::SelectDefense => 'X',
            AttackType::SelectAll => 'A',
        };

        write!(f, "{}", chr)
    }
}
