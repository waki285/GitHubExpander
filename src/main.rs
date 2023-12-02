mod constants;

use std::env;

use octocrab::{
    models::{pulls::{Review, ReviewState}, Author, CommentId, IssueState},
    Octocrab,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serenity::{
    all::{
        ButtonStyle, Client, Context, EventHandler, GatewayIntents, Interaction, Message,
        MessageReference, ReactionType, Ready,
    },
    async_trait,
    builder::{
        CreateActionRow, CreateAllowedMentions, CreateButton, CreateEmbed, CreateEmbedAuthor,
        CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage,
    },
};

use crate::constants::{
    pretty, ISSUE_COMMENT_LINK_REGEX, ISSUE_LINK_REGEX, PR_COMMENT_LINK_REGEX,
    PR_DISCUSSION_LINK_REGEX, PR_LINK_REGEX, PR_REVIEW_LINK_REGEX,
};

#[derive(Debug, Clone, Deserialize)]
struct PullReviewComment {
    user: Author,
    body: Option<String>,
    html_url: Option<String>,
}

static OCTOCRAB: Lazy<Octocrab> = Lazy::new(|| {
    Octocrab::builder()
        .personal_token(
            env::var("GITHUB_TOKEN").expect("Expected a github token in the environment"),
        )
        .build()
        .expect("Failed to create octocrab instance")
});
static OWNER: Lazy<String> = Lazy::new(|| {
    let v = env::var("GITHUB_REPO").unwrap();
    let s = v.split("/").next().unwrap();
    s.to_string()
});
static REPO: Lazy<String> = Lazy::new(|| {
    let v = env::var("GITHUB_REPO").unwrap();
    let s = v.split("/").nth(1).unwrap();
    s.to_string()
});

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _data_about_bot: Ready) {
        log::info!("Connected as {}", ctx.cache.current_user().name);
    }
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }
        let mut embeds: Vec<CreateEmbed> = Vec::new();
        for issue in ISSUE_LINK_REGEX.captures_iter(&msg.content) {
            let issue_id = issue.get(1).unwrap().as_str().parse::<u64>();
            if issue_id.is_err() {
                continue;
            }
            let issue_id = issue_id.unwrap();
            let issue = OCTOCRAB
                .issues(OWNER.clone(), REPO.clone())
                .get(issue_id)
                .await;
            if issue.is_err() {
                continue;
            }
            let issue = issue.unwrap();

            let body = issue
                .body
                .unwrap_or("*No description provided.*".to_string());
            let body = pretty(&body);

            let embed = CreateEmbed::new()
                .title(format!("Issue #{}: {}", issue.number, issue.title))
                .url(issue.html_url)
                .description(body);
            embeds.push(embed);
        }
        for issue_comment in ISSUE_COMMENT_LINK_REGEX.captures_iter(&msg.content) {
            let comment_id = issue_comment.get(2).unwrap().as_str().parse::<u64>();
            if comment_id.is_err() {
                continue;
            }
            let comment_id = comment_id.unwrap();
            let comment = OCTOCRAB
                .issues(OWNER.clone(), REPO.clone())
                .get_comment(CommentId::from(comment_id))
                .await;
            if comment.is_err() {
                continue;
            }
            let comment = comment.unwrap();

            let body = comment
                .body
                .unwrap_or("*No description provided.*".to_string());
            let body = pretty(&body);

            let embed = CreateEmbed::new()
                .url(comment.html_url)
                .author(
                    CreateEmbedAuthor::new(comment.user.login).icon_url(comment.user.avatar_url),
                )
                .description(body);
            embeds.push(embed);
        }
        for pr in PR_LINK_REGEX.captures_iter(&msg.content) {
            let pr_id = pr.get(1).unwrap().as_str().parse::<u64>();
            if pr_id.is_err() {
                continue;
            }
            let pr_id = pr_id.unwrap();
            let pr = OCTOCRAB.pulls(OWNER.clone(), REPO.clone()).get(pr_id).await;
            if pr.is_err() {
                continue;
            }
            let pr = pr.unwrap();

            let body = pr.body.unwrap_or("*No description provided.*".to_string());
            let body = pretty(&body);
            let embed = CreateEmbed::new()
                .title(format!(
                    "PR #{}: {}",
                    pr.number,
                    pr.title.unwrap_or("*No title provided.*".to_string())
                ))
                .url(pr.html_url.unwrap())
                .color({
                    if pr.merged_at.is_some() {
                        0x6f42c1
                    } else if pr.state == Some(IssueState::Open) {
                        0x2cbe4e
                    } else if pr.state == Some(IssueState::Closed) {
                        0xcb2431
                    } else {
                        0x586069
                    }
                })
                .description(body);
            embeds.push(embed);
        }
        for pr_comment in PR_COMMENT_LINK_REGEX.captures_iter(&msg.content) {
            let comment_id = pr_comment.get(2).unwrap().as_str().parse::<u64>();
            if comment_id.is_err() {
                continue;
            }
            let comment_id = comment_id.unwrap();
            let comment = OCTOCRAB
                .issues(OWNER.clone(), REPO.clone())
                .get_comment(CommentId::from(comment_id))
                .await;
            if comment.is_err() {
                continue;
            }
            let comment = comment.unwrap();

            let body = comment
                .body
                .unwrap_or("*No description provided.*".to_string());
            let body = pretty(&body);

            let embed = CreateEmbed::new()
                .url(comment.html_url)
                .author(
                    CreateEmbedAuthor::new(comment.user.login).icon_url(comment.user.avatar_url),
                )
                .description(body);
            embeds.push(embed);
        }
        for pr_discussion in PR_DISCUSSION_LINK_REGEX.captures_iter(&msg.content) {
            let comment_id = pr_discussion.get(2).unwrap().as_str().parse::<u64>();
            if comment_id.is_err() {
                continue;
            }
            let comment_id = comment_id.unwrap();
            let comment = OCTOCRAB
                .get::<PullReviewComment, _, _>(
                    format!(
                        "/repos/{}/{}/pulls/comments/{}",
                        OWNER.to_owned(),
                        REPO.to_owned(),
                        comment_id
                    ),
                    None::<&()>,
                )
                .await;
            if comment.is_err() {
                continue;
            }
            let comment = comment.unwrap();

            let body = comment
                .body
                .unwrap_or("*No description provided.*".to_string());
            let body = pretty(&body);

            let embed = CreateEmbed::new()
                .url(comment.html_url.unwrap())
                .author(
                    CreateEmbedAuthor::new(comment.user.login).icon_url(comment.user.avatar_url),
                )
                .description(body);
            embeds.push(embed);
        }
        for pr_review in PR_REVIEW_LINK_REGEX.captures_iter(&msg.content) {
            let pr_id = pr_review.get(1).unwrap().as_str().parse::<u64>();
            if pr_id.is_err() {
                continue;
            }
            let pr_id = pr_id.unwrap();
            let review_id = pr_review.get(2).unwrap().as_str().parse::<u64>();
            if review_id.is_err() {
                continue;
            }
            let review_id = review_id.unwrap();
            let review = OCTOCRAB
                .get::<Review, _, _>(
                    format!(
                        "/repos/{}/{}/pulls/{}/reviews/{}",
                        OWNER.to_owned(),
                        REPO.to_owned(),
                        pr_id,
                        review_id
                    ),
                    None::<&()>,
                )
                .await;
            if review.is_err() {
                continue;
            }
            let review = review.unwrap();

            let body = review
                .body
                .unwrap_or("*No description provided.*".to_string());
            let body = pretty(&body);

            let embed = CreateEmbed::new()
                .url(review.html_url)
                .author(
                    CreateEmbedAuthor::new(review.user.as_ref().and_then(|f| Some(f.login.clone())).unwrap_or("Unknown User".to_string())).icon_url(review.user.and_then(|f| Some(f.avatar_url.to_string())).unwrap_or("https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png".to_string())),
                )
                .title({
                    let state = review.state.unwrap_or(ReviewState::Pending);
                    if state == ReviewState::Approved {
                        "Approved"
                    } else if state == ReviewState::ChangesRequested {
                        "Changes Requested"
                    } else {
                        "Commented"
                    }
                })
                .description(body)
                .color({
                    let state = review.state.unwrap_or(ReviewState::Pending);
                    if state == ReviewState::Approved {
                        0x2cbe4e
                    } else if state == ReviewState::ChangesRequested {
                        0xcb2431
                    } else {
                        0x586069
                    }
                });
            embeds.push(embed);
        }

        if embeds.len() > 0 {
            let btn = CreateButton::new(format!("del_{}", msg.author.id))
                .emoji(ReactionType::Unicode("🗑️".to_string()))
                .style(ButtonStyle::Secondary);
            msg.channel_id
                .send_message(
                    &ctx.http,
                    CreateMessage::default()
                        .embeds(embeds)
                        .components(vec![CreateActionRow::Buttons(vec![btn])])
                        .reference_message(MessageReference::from(&msg))
                        .allowed_mentions(CreateAllowedMentions::new().replied_user(false)),
                )
                .await
                .unwrap();
        }
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Component(interaction) = interaction {
            if interaction.data.custom_id.starts_with("del_") {
                let allowed_user_id = interaction
                    .data
                    .custom_id
                    .strip_prefix("del_")
                    .unwrap()
                    .parse::<u64>()
                    .unwrap();
                let user_id = interaction.user.id;
                if allowed_user_id != user_id.get() {
                    interaction
                        .create_response(
                            &ctx.http,
                            CreateInteractionResponse::Message(
                                CreateInteractionResponseMessage::new()
                                    .content("メッセージの投稿者のみが削除できます。")
                                    .ephemeral(true),
                            ),
                        )
                        .await
                        .unwrap();
                    return;
                }
                interaction
                    .message
                    .delete(&ctx.http)
                    .await
                    .expect("Failed to delete message");
            }
        }
    }
}

#[inline]
fn get_intents() -> GatewayIntents {
    let mut intents = GatewayIntents::empty();
    intents.insert(GatewayIntents::GUILDS);
    intents.insert(GatewayIntents::GUILD_MESSAGES);
    intents.insert(GatewayIntents::MESSAGE_CONTENT);
    intents
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    env_logger::builder()
        .filter_module("github_expander", {
            if cfg!(debug_assertions) {
                log::LevelFilter::Trace
            } else {
                log::LevelFilter::Info
            }
        })
        .init();

    let discord_token =
        env::var("DISCORD_TOKEN").expect("Expected a discord token in the environment");
    // check exists
    env::var("GITHUB_TOKEN").expect("Expected a github token in the environment");

    log::debug!("Creating client");

    let mut client = Client::builder(discord_token, get_intents())
        .event_handler(Handler)
        .await
        .unwrap();

    log::debug!("Starting client");

    if let Err(why) = client.start().await {
        log::error!("An error occurred while running the client: {:?}", why);
    }
}
