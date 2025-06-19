//! # Difficultly Module for Challenge Handling.

/// This function maps a request's liklihood of being a bot to a difficulty
/// of a proof of work challenge. 
/// Bots are given a score between 1 and 99 of how likely they are to be human.
/// This bot score matches the industry standard used by Cloudflare
/// in the cf.bot_management.score API call
/// A score of 1 is the highest liklihood of being a bot,
/// A score of 99 is the highest liklihood of being human.
pub fn bot_score_to_difficulty(bot_score: u64, base_difficulty: u64, scaling_factor: u64) -> u64 {
    let inverted_score: u64 = 99 - bot_score;
    let difficulty: u64 = ((inverted_score * inverted_score) * scaling_factor) + base_difficulty;
    difficulty
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_score_to_difficulty() {
        assert_eq!(bot_score_to_difficulty(99, 10_000, 1040), 10_000);
        assert_eq!(bot_score_to_difficulty(0, 10_000, 1040), 10_203_040);
        assert_eq!(bot_score_to_difficulty(1, 10_000, 1040), 9_998_160);
    }
}