use std::{collections::HashMap, time::Duration};

use anyhow::{Result, anyhow};
use log::info;
use poise::serenity_prelude as serenity;
use scraper::{ElementRef, Html, Node, Selector};

use crate::Context;

#[derive(Debug)]
pub struct TrackerConfig {
    pub name: &'static str,
    pub url: &'static str,
}

#[derive(Debug, Clone)]
pub struct TrackerRecord {
    name: Box<str>,
    fields: Vec<Box<str>>,
}

#[derive(Debug, Clone, Copy)]
pub struct SearchResult<'a> {
    pub name_raw: &'a str,
    pub name_clean: &'a str,
    pub name_extra: Option<&'a str>,
    pub record: &'a TrackerRecord,
}

#[derive(Debug, Clone)]
pub struct Tracker {
    pub name: &'static str,
    headers: Vec<Box<str>>,
    records: Vec<TrackerRecord>,
    index: Vec<(Box<str>, usize)>,
}

impl Tracker {
    pub async fn build(config: &'static TrackerConfig) -> Result<Self> {
        let html = reqwest::get(config.url)
            .await
            .map_err(|e| anyhow!("[{}] failed to fetch: {e}", config.name))?
            .text()
            .await
            .map_err(|e| anyhow!("[{}] failed to read response body: {e}", config.name))?;

        let (headers, records) = parse_html(&html)
            .map_err(|e| anyhow!("[{}] failed to parse HTML: {e}", config.name))?;

        info!("[{}] {} records", config.name, records.len());

        let index = records
            .iter()
            .enumerate()
            .map(|(i, r)| (Box::from(normalize(&r.name).as_str()), i))
            .collect();

        Ok(Self {
            name: config.name,
            headers,
            records,
            index,
        })
    }

    pub fn search(&self, query: &str) -> Vec<SearchResult<'_>> {
        let normalized = normalize(query);

        if normalized.is_empty() {
            return Vec::new();
        }

        let query_counts = word_counts(&normalized);

        self.index
            .iter()
            .filter(|(name, _)| {
                let name_counts = word_counts(name);

                query_counts.iter().all(|(word, required)| {
                    name_counts.get(word).copied().unwrap_or(0) >= *required
                })
            })
            .map(|(_, i)| {
                let record = &self.records[*i];
                let name_raw = record.name.as_ref();
                let (name_clean, name_extra) =
                    if let Some((clean, extra)) = name_raw.split_once('\n') {
                        (clean, Some(extra))
                    } else {
                        (name_raw, None)
                    };

                SearchResult {
                    name_raw,
                    name_clean,
                    name_extra,
                    record,
                }
            })
            .collect()
    }

    const FOOTER_TEXT: &str = "Made by botmert ♡";
    const FOOTER_ICON: &str = "https://i.imgur.com/8uTFXwR.png"; // my cat

    fn get_footer() -> serenity::CreateEmbedFooter {
        serenity::CreateEmbedFooter::new(Self::FOOTER_TEXT).icon_url(Self::FOOTER_ICON)
    }

    fn build_embed(
        &self,
        results: &[SearchResult<'_>],
        page: usize,
    ) -> anyhow::Result<serenity::CreateEmbed> {
        let result = results
            .get(page)
            .ok_or_else(|| anyhow!("Index out of bounds. {page}"))?;

        Ok(serenity::CreateEmbed::new()
            .title(result.name_clean)
            .description(result.name_extra.unwrap_or(""))
            .fields(
                self.headers
                    .iter()
                    .zip(&result.record.fields)
                    .filter_map(|(header, value)| {
                        if value.trim() == "" {
                            return None;
                        }
                        Some((header.as_ref(), value.as_ref(), false))
                    }),
            )
            .color(0x00F55E)
            .footer(Self::get_footer()))
    }

    const BTN_PREV: &str = "page_prev";
    const BTN_NEXT: &str = "page_next";
    const BTN_CHOOSE: &str = "page_choose";
    const BTN_FIRST: &str = "page_first";
    const BTN_LAST: &str = "page_last";

    fn create_nav_row(
        page: usize,
        total_pages: usize,
        disabled: bool,
    ) -> serenity::CreateActionRow {
        let at_first = disabled || page == 0;
        let at_last = disabled || page + 1 >= total_pages;

        serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(Self::BTN_FIRST)
                .label("First")
                .style(serenity::ButtonStyle::Secondary)
                .disabled(at_first),
            serenity::CreateButton::new(Self::BTN_PREV)
                .label("Back")
                .style(serenity::ButtonStyle::Secondary)
                .disabled(at_first),
            serenity::CreateButton::new(Self::BTN_CHOOSE)
                .label(format!("{page}/{total_pages}", page = page + 1))
                .style(serenity::ButtonStyle::Secondary)
                .disabled(disabled),
            serenity::CreateButton::new(Self::BTN_NEXT)
                .label("Next")
                .style(serenity::ButtonStyle::Secondary)
                .disabled(at_last),
            serenity::CreateButton::new(Self::BTN_LAST)
                .label("Last")
                .style(serenity::ButtonStyle::Secondary)
                .disabled(at_last),
        ])
    }

    fn enabled_nav_row(page: usize, total_pages: usize) -> serenity::CreateActionRow {
        Self::create_nav_row(page, total_pages, false)
    }

    fn disabled_nav_row() -> serenity::CreateActionRow {
        Self::create_nav_row(0, 0, true)
    }

    const MODAL_CHOOSE: &str = "modal_choose";

    pub async fn send_modal<'a>(
        &self,
        ctx: Context<'a>,
        interaction: &serenity::model::application::ComponentInteraction,
        total_pages: usize,
    ) -> Result<Option<usize>> {
        let field_id = format!("modal_page_{}", interaction.id);

        let modal = serenity::CreateModal::new(Self::MODAL_CHOOSE, "Choose Page").components(vec![
            serenity::CreateActionRow::InputText(
                serenity::CreateInputText::new(serenity::InputTextStyle::Short, "Page", &field_id)
                    .placeholder("e.g. 1")
                    .min_length(1)
                    .max_length(6)
                    .required(true),
            ),
        ]);

        interaction
            .create_response(
                ctx.http(),
                serenity::CreateInteractionResponse::Modal(modal),
            )
            .await?;

        let Some(submit) = interaction
            .message
            .await_modal_interaction(ctx)
            .author_id(interaction.user.id)
            .custom_ids(vec![Self::MODAL_CHOOSE.to_string()])
            .timeout(Duration::from_secs(300))
            .await
        else {
            return Ok(None);
        };

        let value = submit
            .data
            .components
            .iter()
            .flat_map(|row| row.components.iter())
            .find_map(|component| match component {
                serenity::ActionRowComponent::InputText(input) if input.custom_id == field_id => {
                    input.value.as_deref()
                }
                _ => None,
            })
            .ok_or_else(|| anyhow!("missing page input"))?;

        let page_1_based: usize = value
            .trim()
            .parse()
            .map_err(|_| anyhow!("page must be a number"))?;

        if page_1_based == 0 || page_1_based > total_pages {
            submit
                .create_response(
                    ctx.http(),
                    serenity::CreateInteractionResponse::Message(
                        serenity::CreateInteractionResponseMessage::new()
                            .content(format!("Page must be between 1 and {total_pages}."))
                            .ephemeral(true),
                    ),
                )
                .await?;
            Ok(None)
        } else {
            submit
                .create_response(ctx.http(), serenity::CreateInteractionResponse::Acknowledge)
                .await?;
            Ok(Some(page_1_based - 1))
        }
    }

    pub async fn send_embed<'a>(
        &self,
        ctx: Context<'a>,
        query: &str,
        results: &[SearchResult<'_>],
    ) -> Result<()> {
        let total_pages = results.len();
        let mut page = 0usize;

        if results.is_empty() {
            let error = serenity::CreateEmbed::new()
                .title("Error")
                .description(format!("No result found for \"{query}\""))
                .color(serenity::Color::RED)
                .footer(Self::get_footer());

            ctx.send(
                poise::CreateReply::default()
                    .reply(true)
                    .allowed_mentions(serenity::CreateAllowedMentions::new().replied_user(false))
                    .embed(error),
            )
            .await?;

            return Ok(());
        }

        let reply = ctx
            .send(
                poise::CreateReply::default()
                    .reply(true)
                    .allowed_mentions(serenity::CreateAllowedMentions::new().replied_user(false))
                    .embed(self.build_embed(results, page)?)
                    .components(vec![Self::enabled_nav_row(page, total_pages)]),
            )
            .await?;

        let mut search_msg = reply.into_message().await?;

        loop {
            let Some(interaction) = serenity::collector::ComponentInteractionCollector::new(ctx)
                .message_id(search_msg.id)
                .author_id(ctx.author().id)
                .timeout(Duration::from_secs(600))
                .await
            else {
                search_msg
                    .edit(
                        ctx,
                        serenity::EditMessage::new().components(vec![Self::disabled_nav_row()]),
                    )
                    .await?;
                break;
            };

            if interaction.data.custom_id == Self::BTN_CHOOSE {
                if let Some(new_page) = self.send_modal(ctx, &interaction, total_pages).await? {
                    if new_page != page {
                        page = new_page;
                        search_msg
                            .edit(
                                ctx,
                                serenity::EditMessage::new()
                                    .embed(self.build_embed(results, page)?)
                                    .components(vec![Self::enabled_nav_row(page, total_pages)]),
                            )
                            .await?;
                    }
                }
                continue;
            }

            let new_page = match interaction.data.custom_id.as_str() {
                Self::BTN_FIRST => 0,
                Self::BTN_PREV => page.saturating_sub(1),
                Self::BTN_NEXT => (page + 1).min(total_pages - 1),
                Self::BTN_LAST => total_pages - 1,
                _ => page,
            };

            let prev_page = page;
            page = new_page;

            interaction
                .create_response(ctx.http(), serenity::CreateInteractionResponse::Acknowledge)
                .await?;

            if page != prev_page {
                search_msg
                    .edit(
                        ctx,
                        serenity::EditMessage::new()
                            .embed(self.build_embed(results, page)?)
                            .components(vec![Self::enabled_nav_row(page, total_pages)]),
                    )
                    .await?;
            }
        }

        Ok(())
    }
}

fn normalize(s: &str) -> String {
    fn is_quote(ch: char) -> bool {
        matches!(
            ch,
            '\'' | '"'
                | '`'
                | '\u{2018}'
                | '\u{2019}'
                | '\u{201A}'
                | '\u{201B}'
                | '\u{201C}'
                | '\u{201D}'
                | '\u{201E}'
                | '\u{201F}'
                | '\u{02BC}'
                | '\u{02B9}'
                | '\u{02C8}'
                | '\u{FF07}'
                | '\u{FF02}'
        )
    }

    let mut out = String::with_capacity(s.len());
    let mut needs_space = false;

    for c in s.chars() {
        if is_quote(c) {
            continue;
        }
        if c.is_alphanumeric() {
            if needs_space && !out.is_empty() {
                out.push(' ');
            }
            for lc in c.to_lowercase() {
                out.push(lc);
            }
            needs_space = false;
        } else {
            if !out.is_empty() {
                needs_space = true;
            }
        }
    }

    out
}

fn parse_html(html: &str) -> Result<(Vec<Box<str>>, Vec<TrackerRecord>)> {
    let document = Html::parse_fragment(html);
    let tr_selector = Selector::parse("tr").unwrap();
    let td_selector = Selector::parse("td").unwrap();

    let mut rows = document.select(&tr_selector);

    let header_row = rows
        .find(|row| row.select(&td_selector).next().is_some())
        .ok_or_else(|| anyhow!("no header row found, is the page returning an error page?"))?;

    let mut headers: Vec<Box<str>> = header_row
        .select(&td_selector)
        .map(|cell| Box::from(header_text(cell).as_str()))
        .collect();

    if headers.is_empty() {
        return Err(anyhow!("header row found but contained no <td> elements"));
    }

    // TODO: More variants?
    let (name_key_index, _) = headers
        .iter()
        .enumerate()
        .find(|(_, h)| h.starts_with("Name"))
        .ok_or_else(|| anyhow!("no key starting with 'Name' found; headers: {headers:?}"))?;

    info!("Headers: {headers:?}, name key index: {name_key_index:?}");

    let mut records = Vec::with_capacity(5_000);

    // TODO: Use iter zip?

    for row in rows {
        let cols: Vec<String> = row.select(&td_selector).map(cell_text).collect();

        if cols.len() <= 8 || cols.iter().all(|s| s.is_empty()) {
            continue;
        }

        let mut fields: Vec<Box<str>> = headers
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let val = cols.get(i).map(|s| s.as_str()).unwrap_or("");
                Box::from(val)
            })
            .collect();

        let name = fields[name_key_index].clone();
        fields.remove(name_key_index);

        records.push(TrackerRecord { name, fields });
    }

    if records.is_empty() {
        return Err(anyhow!("parsed headers but found no data rows"));
    }

    headers.remove(name_key_index);

    info!("Parsed {} records", records.len());
    Ok((headers, records))
}

fn collect_text(element: ElementRef<'_>, out: &mut String, br: char) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => out.push_str(text),
            Node::Element(elem) if elem.name() == "br" => out.push(br),
            Node::Element(_) => {
                if let Some(child_elem) = ElementRef::wrap(child) {
                    collect_text(child_elem, out, br);
                }
            }
            _ => {}
        }
    }
}

fn header_text(cell: ElementRef<'_>) -> String {
    let mut raw = String::new();
    collect_text(cell, &mut raw, ' ');

    let raw = raw.split_once(" (").map_or(raw.as_str(), |(head, _)| head);
    process_line(raw)
}

fn cell_text(cell: ElementRef<'_>) -> String {
    let mut raw = String::new();
    collect_text(cell, &mut raw, '\n');

    raw.lines().map(process_line).collect::<Vec<_>>().join("\n")
}

fn process_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut pending_space = false;

    for c in line.chars() {
        if c.is_whitespace() {
            pending_space = true;
            continue;
        }

        if pending_space
            && !out.is_empty()
            && matches!(c, '.' | ',' | '!' | '?' | ':' | ';' | ')' | ']' | '}')
        {
            pending_space = false;
        } else if pending_space && !out.is_empty() {
            out.push(' ');
            pending_space = false;
        }

        out.push(c);
    }

    out
}

fn word_counts(s: &str) -> HashMap<&str, usize> {
    let mut counts = HashMap::new();

    for word in s.split_whitespace() {
        *counts.entry(word).or_insert(0) += 1;
    }

    counts
}
