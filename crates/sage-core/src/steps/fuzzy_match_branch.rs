use anyhow::Result;
use fuzzy_matcher::{FuzzyMatcher, skim::SkimMatcherV2};

pub fn fuzzy_match_branch(name: &str, branches: Vec<String>) -> Result<Option<String>> {
    let matcher = SkimMatcherV2::default();

    let mut best_match = None;
    let mut best_score = 0;

    // First check for exact match (case insensitive)
    for branch in &branches {
        if branch.eq_ignore_ascii_case(name) {
            return Ok(Some(branch.clone()));
        }
    }

    // If no exact match, perform fuzzy search
    for branch in &branches {
        if let Some(score) = matcher.fuzzy_match(branch, name)
            && score > best_score
        {
            best_score = score;
            best_match = Some(branch.clone());
        }
    }

    // use the best match if found
    if let Some(best_match) = best_match {
        return Ok(Some(best_match));
    }

    Ok(None)
}
