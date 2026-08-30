use crate::models::card::Card;

mod assets;
mod models;

fn main() {
    let card = Card::new(
        87,
        4,
        String::from("Flan"),
        0x67,
        5,
        3,
        10,
        models::battle_class::BattleClass::Physical,
    );

    println!("{}", card);
}
