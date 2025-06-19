//! # Difficultly Module for Challenge Handling.

fn bot_score_to_difficulty(bot_score: u64) -> u64 {
    let inverted_score: u64 = 99 - bot_score;
    let base_difficulty: u64 = 10_000;
    let difficulty: u64 = ((inverted_score * inverted_score) * 1040) + base_difficulty;
    difficulty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_score_to_difficulty() {
        assert_eq!(bot_score_to_difficulty(99), 10_000);
        assert_eq!(bot_score_to_difficulty(0), 10_203_040);
        assert_eq!(bot_score_to_difficulty(1), 9_998_160);
    }
}