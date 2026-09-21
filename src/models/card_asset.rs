use crate::{
    assets::card_record::CardRecord,
    models::{battle_class::BattleClass, card_stats::CardStats},
};

#[derive(PartialEq, Eq)]
pub struct CardAsset {
    pub index: usize,
    pub name: String,
    pub base_stats: CardStats,
}

impl CardAsset {
    fn parse_stat(c_opt: Option<char>) -> Result<u8, String> {
        let c = c_opt.ok_or("unable to read char")?;
        let parsed = c.to_digit(16).ok_or(format!("Invalid stat value {}", c))?;
        let stat = (parsed << 4) + 0xf;
        Ok(stat as u8)
    }

    fn parse_battle_class(c_opt: Option<char>) -> Result<BattleClass, String> {
        let c = c_opt.ok_or("unable to read char")?;
        match c {
            'P' => Ok(BattleClass::Physical),
            'M' => Ok(BattleClass::Magic),
            'X' => Ok(BattleClass::Flexible),
            'A' => Ok(BattleClass::Assault),
            _ => Err(format!("Invalid battle class:  {}", c)),
        }
    }
}

impl TryFrom<(usize, CardRecord)> for CardAsset {
    type Error = String;

    fn try_from((index, record): (usize, CardRecord)) -> Result<Self, Self::Error> {
        let mut iter = record.stats.chars();
        let attack = Self::parse_stat(iter.next())?;
        let battle_class = Self::parse_battle_class(iter.next())?;
        let phys_defense = Self::parse_stat(iter.next())?;
        let mag_defense = Self::parse_stat(iter.next())?;

        Ok(Self {
            index,
            name: record.name,
            base_stats: CardStats {
                attack,
                battle_class,
                phys_defense,
                mag_defense,
            },
        })
    }
}
