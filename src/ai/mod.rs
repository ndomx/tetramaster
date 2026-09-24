use crate::{
    models::{
        core::{board::Board, card::Card, geometry::Position},
        session::GameAction,
    },
    utils::random::GameRng,
};

pub struct CpuMoveInput<'a> {
    pub board: &'a Board,
    pub hand: &'a [Card],
    pub legal_actions: &'a [GameAction],
}

pub fn choose_random_action(input: CpuMoveInput<'_>, rng: &mut GameRng) -> Option<GameAction> {
    let positions = input
        .legal_actions
        .iter()
        .map(|action| action.position())
        .fold(Vec::<Position>::new(), |mut positions, target| {
            if !positions.contains(&target) {
                positions.push(target);
            }
            positions
        });
    let target = rng.choose(&positions).copied()?;
    let actions = input
        .legal_actions
        .iter()
        .filter(|action| action.position() == target)
        .collect::<Vec<_>>();

    rng.choose(&actions).copied().copied()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        models::core::{board::Board, card::BattleClass, geometry::Position},
        test_support::{card, empty_tiles},
    };

    fn input<'a>(
        board: &'a Board,
        hand: &'a [Card],
        legal_actions: &'a [GameAction],
    ) -> CpuMoveInput<'a> {
        CpuMoveInput {
            board,
            hand,
            legal_actions,
        }
    }

    #[test]
    fn random_policy_returns_only_a_supplied_legal_action() {
        let board = Board::from_tiles(empty_tiles());
        let hand = vec![card(10, 0, BattleClass::Physical, 1, 1, 1)];
        let legal_actions = vec![
            GameAction::new(10, Position::new(0, 1)),
            GameAction::new(10, Position::new(2, 3)),
        ];

        for seed in 0..32 {
            let chosen = choose_random_action(
                input(&board, &hand, &legal_actions),
                &mut GameRng::from_seed(seed),
            )
            .unwrap();
            assert!(legal_actions.contains(&chosen));
        }
    }

    #[test]
    fn random_policy_is_reproducible_for_a_seed() {
        let board = Board::from_tiles(empty_tiles());
        let hand = vec![
            card(10, 0, BattleClass::Physical, 1, 1, 1),
            card(20, 0, BattleClass::Physical, 1, 1, 1),
        ];
        let legal_actions = vec![
            GameAction::new(10, Position::new(0, 0)),
            GameAction::new(20, Position::new(0, 0)),
            GameAction::new(10, Position::new(3, 3)),
            GameAction::new(20, Position::new(3, 3)),
        ];
        let mut first_rng = GameRng::from_seed(0xA1_CAFE);
        let mut second_rng = GameRng::from_seed(0xA1_CAFE);

        let first = choose_random_action(input(&board, &hand, &legal_actions), &mut first_rng);
        let second = choose_random_action(input(&board, &hand, &legal_actions), &mut second_rng);

        assert_eq!(first, second);
    }

    #[test]
    fn random_policy_returns_none_without_legal_actions() {
        let board = Board::from_tiles(empty_tiles());
        let hand = Vec::new();

        assert_eq!(
            choose_random_action(input(&board, &hand, &[]), &mut GameRng::from_seed(1)),
            None
        );
    }
}
