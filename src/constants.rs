use std::env;

use once_cell::sync::Lazy;
use regex::Regex;

pub static ISSUE_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        format!(
            r"https:\/\/github.com\/{}/issues\/(\d+)",
            env::var("GITHUB_REPO").unwrap().replace("/", r"\/")
        )
        .as_str(),
    )
    .expect("Failed to create issue link regex")
});

pub static ISSUE_COMMENT_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        format!(
            r"https:\/\/github.com\/{}/issues\/(\d+)#issuecomment-(\d+)",
            env::var("GITHUB_REPO").unwrap().replace("/", r"\/")
        )
        .as_str(),
    )
    .expect("Failed to create issue comment link regex")
});

pub static PR_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        format!(
            r"https:\/\/github.com\/{}/pull\/(\d+)",
            env::var("GITHUB_REPO").unwrap().replace("/", r"\/")
        )
        .as_str(),
    )
    .expect("Failed to create PR link regex")
});

pub static PR_COMMENT_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        format!(
            r"https:\/\/github.com\/{}/pull\/(\d+)#issuecomment-(\d+)",
            env::var("GITHUB_REPO").unwrap().replace("/", r"\/")
        )
        .as_str(),
    )
    .expect("Failed to create PR comment link regex")
});

pub static PR_DISCUSSION_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        format!(
            r"https:\/\/github.com\/{}/pull\/(\d+)#discussion_r(\d+)",
            env::var("GITHUB_REPO").unwrap().replace("/", r"\/")
        )
        .as_str(),
    )
    .expect("Failed to create PR discussion link regex")
});

pub static PR_REVIEW_LINK_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        format!(
            r"https:\/\/github.com\/{}/pull\/(\d+)#pullrequestreview-(\d+)",
            env::var("GITHUB_REPO").unwrap().replace("/", r"\/")
        )
        .as_str(),
    )
    .expect("Failed to create PR review link regex")
});

pub static HTML_COMMENT_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?s)<!--.*?-->").unwrap());
pub static CHECKBOX_BLANK_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\s*-\s+\[ ]").unwrap());
pub static CHECKBOX_CHECKED_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\s*-\s+\[x]").unwrap());
pub static MULTILINE_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(\r\n|\n){3,}").unwrap());

#[inline]
pub fn pretty(body: &str) -> String {
    let body = HTML_COMMENT_REGEX.replace_all(body, "");
    // チェックボックスを絵文字に置換
    let body = CHECKBOX_BLANK_REGEX
        .replace_all(&body, "- :white_square_button:")
        .to_string();
    let body = CHECKBOX_CHECKED_REGEX
        .replace_all(&body, "- :white_check_mark:")
        .to_string();
    // 連続する改行を2つにする
    let body = MULTILINE_REGEX.replace_all(&body, "\n\n").to_string();
    body
}
