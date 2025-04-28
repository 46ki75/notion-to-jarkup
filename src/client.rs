use std::any::Any;

use jarkup_rs::InlineComponent;

#[derive(Debug)]
pub struct Client {
    notionrs_client: notionrs::client::Client,
    reqwest_client: reqwest::Client,
}

impl Client {
    pub async fn convert_block(
        &self,
        block_id: &str,
    ) -> Result<Vec<jarkup_rs::Component>, crate::error::Error> {
        let components: Vec<jarkup_rs::Component> = Vec::new();

        let blocks = self
            .notionrs_client
            .get_block_children_all()
            .block_id(block_id)
            .send()
            .await?;

        for block in blocks {
            match block.block {
                notionrs::object::block::Block::Audio { audio: _ } => todo!(),
                notionrs::object::block::Block::Bookmark { bookmark } => todo!(),
                notionrs::object::block::Block::Breadcrumb { breadcrumb } => todo!(),
                notionrs::object::block::Block::BulletedListItem { bulleted_list_item } => todo!(),
                notionrs::object::block::Block::Callout { callout } => todo!(),
                notionrs::object::block::Block::ChildDatabase { child_database } => todo!(),
                notionrs::object::block::Block::ChildPage { child_page } => todo!(),
                notionrs::object::block::Block::Code { code } => {
                    // let component = jarkup_rs::CodeBlock {
                    //     inline: false,
                    //     props: jarkup_rs::CodeBlockProps {
                    //         code: code
                    //             .rich_text
                    //             .into_iter()
                    //             .map(|r| r.to_string())
                    //             .collect::<Vec<String>>()
                    //             .join(""),
                    //         language: code.language.to_string(),
                    //     },
                    //     slots: todo!(),
                    // };
                    // components.push(component.into());
                    todo!()
                }
                notionrs::object::block::Block::ColumnList { column_list } => todo!(),
                notionrs::object::block::Block::Column { column } => todo!(),
                notionrs::object::block::Block::Divider { divider } => todo!(),
                notionrs::object::block::Block::Embed { embed } => todo!(),
                notionrs::object::block::Block::Equation { equation } => todo!(),
                notionrs::object::block::Block::File { file } => todo!(),
                notionrs::object::block::Block::Heading1 { heading_1 } => todo!(),
                notionrs::object::block::Block::Heading2 { heading_2 } => todo!(),
                notionrs::object::block::Block::Heading3 { heading_3 } => todo!(),
                notionrs::object::block::Block::Image { image } => todo!(),
                notionrs::object::block::Block::LinkPreview { link_preview } => todo!(),
                notionrs::object::block::Block::NumberedListItem { numbered_list_item } => todo!(),
                notionrs::object::block::Block::Paragraph { paragraph } => todo!(),
                notionrs::object::block::Block::Pdf { pdf } => todo!(),
                notionrs::object::block::Block::Quote { quote } => todo!(),
                notionrs::object::block::Block::SyncedBlock { synced_block } => todo!(),
                notionrs::object::block::Block::TableOfContents { table_of_contents } => todo!(),
                notionrs::object::block::Block::Table { table } => todo!(),
                notionrs::object::block::Block::TableRow { table_row } => todo!(),
                notionrs::object::block::Block::Template { template } => todo!(),
                notionrs::object::block::Block::ToDo { to_do } => todo!(),
                notionrs::object::block::Block::Toggle { toggle } => todo!(),
                notionrs::object::block::Block::Video { video } => todo!(),
                notionrs::object::block::Block::Unsupported => todo!(),
            }
        }

        todo!()
    }

    pub async fn convert_rich_text(
        &self,
        rich_text_vec: Vec<notionrs::object::rich_text::RichText>,
    ) -> Result<Vec<jarkup_rs::InlineComponent>, crate::error::Error> {
        let mut components: Vec<jarkup_rs::InlineComponent> = Vec::new();

        for rich_text in rich_text_vec {
            let component: Result<jarkup_rs::InlineComponent, crate::error::Error> = match rich_text
            {
                notionrs::object::rich_text::RichText::Text {
                    text,
                    annotations,
                    plain_text,
                    href,
                } => {
                    let component = jarkup_rs::Text {
                        inline: true,
                        props: jarkup_rs::TextProps {
                            text: plain_text,
                            color: match annotations.color {
                                notionrs::object::color::Color::Default => None,
                                notionrs::object::color::Color::Blue => {
                                    Some(String::from("#6987b8"))
                                }
                                notionrs::object::color::Color::Brown => {
                                    Some(String::from("#8b4c3f"))
                                }
                                notionrs::object::color::Color::Gray => {
                                    Some(String::from("#868e9c"))
                                }
                                notionrs::object::color::Color::Green => {
                                    Some(String::from("#59b57c"))
                                }
                                notionrs::object::color::Color::Orange => {
                                    Some(String::from("#bf7e71"))
                                }
                                notionrs::object::color::Color::Pink => {
                                    Some(String::from("#c9699e"))
                                }
                                notionrs::object::color::Color::Purple => {
                                    Some(String::from("#9771bd"))
                                }
                                notionrs::object::color::Color::Red => {
                                    Some(String::from("#b36472"))
                                }
                                notionrs::object::color::Color::Yellow => {
                                    Some(String::from("#b8a36e"))
                                }
                                _ => None,
                            },
                            background_color: match annotations.color {
                                notionrs::object::color::Color::Default => None,
                                notionrs::object::color::Color::BlueBackground => {
                                    Some(String::from("#6987b8"))
                                }
                                notionrs::object::color::Color::BrownBackground => {
                                    Some(String::from("#8b4c3f"))
                                }
                                notionrs::object::color::Color::GrayBackground => {
                                    Some(String::from("#868e9c"))
                                }
                                notionrs::object::color::Color::GreenBackground => {
                                    Some(String::from("#59b57c"))
                                }
                                notionrs::object::color::Color::OrangeBackground => {
                                    Some(String::from("#bf7e71"))
                                }
                                notionrs::object::color::Color::PinkBackground => {
                                    Some(String::from("#c9699e"))
                                }
                                notionrs::object::color::Color::PurpleBackground => {
                                    Some(String::from("#9771bd"))
                                }
                                notionrs::object::color::Color::RedBackground => {
                                    Some(String::from("#b36472"))
                                }
                                notionrs::object::color::Color::YellowBackground => {
                                    Some(String::from("#b8a36e"))
                                }
                                _ => None,
                            },
                            bold: Some(annotations.bold),
                            italic: Some(annotations.italic),
                            underline: Some(annotations.underline),
                            strikethrough: Some(annotations.strikethrough),
                            katex: None,
                            code: Some(annotations.code),
                            ruby: None,
                            favicon: if let Some(l) = &text.link {
                                self.fetch_favicon_by_url(&l.url).await
                            } else {
                                None
                            },
                            href: text.link.map(|l| l.url),
                        },
                        slots: None,
                    };

                    Ok(component.into())
                }
                notionrs::object::rich_text::RichText::Mention {
                    mention,
                    annotations: _annotations,
                    plain_text,
                    href: _href,
                } => {
                    let component: Result<jarkup_rs::InlineComponent, crate::error::Error> =
                        match mention {
                            notionrs::object::rich_text::mention::Mention::User { user: _ } => {
                                continue;
                            }
                            notionrs::object::rich_text::mention::Mention::Date { date: _ } => {
                                continue;
                            }
                            notionrs::object::rich_text::mention::Mention::LinkPreview {
                                link_preview: _,
                            } => {
                                continue;
                            }
                            notionrs::object::rich_text::mention::Mention::LinkMention {
                                link_mention,
                            } => {
                                let component = jarkup_rs::Text {
                                    inline: true,
                                    props: jarkup_rs::TextProps {
                                        text: plain_text,
                                        favicon: self
                                            .fetch_favicon_by_url(&link_mention.href)
                                            .await,
                                        href: Some(link_mention.href),
                                        ..Default::default()
                                    },
                                    slots: None,
                                };

                                let inline_component: jarkup_rs::InlineComponent = component.into();

                                Ok(inline_component)
                            }
                            notionrs::object::rich_text::mention::Mention::TemplateMention {
                                template_mention: _,
                            } => {
                                continue;
                            }
                            notionrs::object::rich_text::mention::Mention::Page { page: _ } => {
                                continue;
                            }
                            notionrs::object::rich_text::mention::Mention::Database {
                                database: _,
                            } => {
                                continue;
                            }
                            notionrs::object::rich_text::mention::Mention::CustomEmoji {
                                custom_emoji,
                            } => {
                                let component = jarkup_rs::Icon {
                                    inline: true,
                                    props: jarkup_rs::IconProps {
                                        src: custom_emoji.url,
                                        alt: Some(custom_emoji.name),
                                    },
                                    slots: None,
                                };

                                let inline_component: jarkup_rs::InlineComponent = component.into();

                                Ok(inline_component)
                            }
                        };

                    component
                }
                notionrs::object::rich_text::RichText::Equation {
                    equation,
                    annotations: _annotations,
                    plain_text: _plain_text,
                    href: _href,
                } => {
                    let component = jarkup_rs::Text {
                        inline: true,
                        props: jarkup_rs::TextProps {
                            text: equation.expression,
                            katex: Some(true),
                            ..Default::default()
                        },
                        slots: None,
                    };

                    Ok(component.into())
                }
            };
        }

        return Ok(components);
    }

    pub(crate) async fn fetch_favicon_by_url(&self, url: &str) -> Option<String> {
        let res = self.reqwest_client.get(url).send().await.ok()?;
        let html = res.text().await.ok()?;
        let meta_scraper = html_meta_scraper::MetaScraper::new(&html);
        meta_scraper.favicon()
    }
}
