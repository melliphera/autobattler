//! As a reminder, movement follows the following principles. Each of these tests presents one of those principles and ensures they work.
//! 
//! 0) walk directly to target if aligned and unblocked.
//! 1) check if orthogonal brings into range and is unblocked
//! 2) check if diagonal brings into range and is unblocked
//! 3) use dot product to see if orthogonal or diagonal is better assuming both unblocked
//! 4) look at all unblocked tiles that decrease either x_dist or y_dist and pick best by alignment.

#[cfg(test)]
pub mod movement_tests {
    use crate::prelude::BattleState;
    use crate::core_structs::prelude::*;

    fn from_teams_to_move(humans: &[(u16, (i32, i32))], enemies: &[(u16, (i32, i32))]) -> GridPosition {
        let mut b = BattleState::new_with_teams(humans, enemies);

        b.initialize();
        let move_event = b.step_event()[0].0;
        if let MoveEvent(data) = move_event {
            println!("{}", data.end_pos);
            return data.end_pos
        } else {
            panic!("Expected MoveEvent, instead got {:#?}", move_event)
        }
    }

    #[test]
    fn out_of_range_direct() {
        // expects resolution 0 because both x are the same

        let human_team = &[(0, (2, 2))]; // knight
        let enemy_team = &[(0, (6, 2))]; // slime

        assert_eq!(
            from_teams_to_move(human_team, enemy_team),
            GridPosition { x: 3, y: 2 }  // knight moves 1 tile towards slime
        )
    }

    #[test]
    fn step_into_orthogonally() {
        // expects resolution 1 - units are just over 6 tiles apart, but 1 orthogonal move away.

        let human_team = &[(1, (1, 3))]; // ranger; 6 range
        let enemy_team = &[(0, (7, 2))]; // slime

        assert_eq!(
            from_teams_to_move(human_team, enemy_team),
            GridPosition { x: 2, y: 3 }  // ranger moves 1 tile towards slime orthogonally and is then in range.
        )
    }

    #[test]
    fn step_into_diagonally() {
        // expects resolution 2
        let human_team = &[(0, (5, 0))]; // knight
        let enemy_team = &[(0, (6, 2))]; // slime

        assert_eq!(
            from_teams_to_move(human_team, enemy_team),
            GridPosition { x: 6, y: 1 }  // knight sees a diagonal tile in range, and moves to it. Orthogonal is out of range.
        )
    }

    #[test]
    fn orthogonal_by_dot() {
        // expects resolution 3
        let human_team = &[(0, (1, 1))]; // knight
        let enemy_team = &[(0, (6, 2))]; // slime

        assert_eq!(
            from_teams_to_move(human_team, enemy_team),
            GridPosition { x: 2, y: 1 }  // with neither orthog or diag in range, chooses orthog bc its closer to line.
        )
    }

    #[test]
    fn diagonal_by_dot() {
        // expects resolution 3 but diagonal
        let human_team = &[(0, (0, 0))]; // knight
        let enemy_team = &[(0, (5, 4))]; // slime

        assert_eq!(
            from_teams_to_move(human_team, enemy_team),
            GridPosition { x: 1, y: 1 }  // out of range, knight calculates diagonal dot product as superior.
        )
    }

    #[test]
    fn perpendicular_blocked() {
        // expects resolution 4 - both best_ortho and best_diag blocked so it moves the other orthogonal direction.
        let human_team = &[
            (0, (2, 1)), // knight

            (0, (3, 1)), // knight used for blocking
            (0, (3, 2)), // knight used for blocking

        ]; 
        let enemy_team = &[
            (0, (5, 3))
        ]; // slime

        assert_eq!(
            from_teams_to_move(human_team, enemy_team),
            GridPosition { x: 2, y: 2 }  // knight moves 1 tile towards slime
        )
    }

    #[test]
    fn stationary_blocked() {
        // all forwards directions blocked so unit doesn't move
        let human_team = &[
            (0, (2, 2)), // knight

            (0, (3, 1)), // knight used for blocking
            (0, (3, 2)), // knight used for blocking
            (0, (3, 3)), // knight used for blocking

        ]; 
        let enemy_team = &[
            (0, (5, 2))
        ]; // slime

        assert_eq!(
            from_teams_to_move(human_team, enemy_team),
            GridPosition { x: 2, y: 2 }  // knight moves 1 tile towards slime
        )
    }

    #[test]
    fn further_because_of_blockage() {
        // unit paths around blockage in a way that actually increases distance to target, because it decreases one of the distances but increases the other
        let human_team = &[
            (0, (2, 0)), // knight

            (0, (3, 0)), // knight used for blocking
            (0, (3, 1)), // knight used for blocking
            (0, (2, 1)), // knight used for blocking

        ]; 
        let enemy_team = &[
            (0, (7, 1))
        ]; // slime

        assert_eq!(
            from_teams_to_move(human_team, enemy_team),
            GridPosition { x: 1, y: 1 }  // knight moves backwards to go around blockade, becaues y 0->1
        )
    }
}