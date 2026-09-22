use crate::{
    assets::card_record::CardRecord,
    models::{battle_class::BattleClass, card_stats::CardStats},
};

#[derive(PartialEq, Eq)]
pub struct CardDefinition {
    pub index: usize,
    pub name: String,
    pub base_stats: CardStats,
}

impl CardDefinition {
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

impl TryFrom<(usize, CardRecord)> for CardDefinition {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn record(stats: &str) -> CardRecord {
        CardRecord {
            name: "Test".into(),
            stats: stats.into(),
        }
    }

    #[test]
    fn parses_hex_stats_and_each_battle_class() {
        for (letter, class) in [
            ('P', BattleClass::Physical),
            ('M', BattleClass::Magic),
            ('X', BattleClass::Flexible),
            ('A', BattleClass::Assault),
        ] {
            let asset = CardDefinition::try_from((3, record(&format!("A{letter}3F")))).unwrap();
            assert_eq!(asset.index, 3);
            assert_eq!(asset.base_stats.attack, 0xaf);
            assert_eq!(asset.base_stats.battle_class, class);
            assert_eq!(asset.base_stats.phys_defense, 0x3f);
            assert_eq!(asset.base_stats.mag_defense, 0xff);
        }
    }

    #[test]
    fn rejects_missing_invalid_or_unknown_stat_fields() {
        assert!(CardDefinition::try_from((0, record("0P0"))).is_err());
        assert!(CardDefinition::try_from((0, record("GP00"))).is_err());
        assert!(CardDefinition::try_from((0, record("0Q00"))).is_err());
    }
}
