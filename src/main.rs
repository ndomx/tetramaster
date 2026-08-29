use crate::models::card::Card;

mod models;

fn main() {
    let card = Card::new(
        87,
        String::from("Flan"),
        0x67,
        5,
        3,
        10,
        models::attack_type::AttackType::Physical,
    );

    println!("{}", card);
}
