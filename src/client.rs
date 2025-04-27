use std::any::Any;

use jarkup_rs::InlineComponent;

#[derive(Debug)]
pub struct Client {
    notionrs_client: notionrs::client::Client,
}

#[derive(Debug, Default)]
pub struct PageMeta {
    url: String,
    title: Option<String>,
    description: Option<String>,
    image: Option<String>,
    favicon: Option<String>,
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

    async fn convert_rich_text(
        &self,
        rich_text: notionrs::object::rich_text::RichText,
    ) -> Result<jarkup_rs::InlineComponent, crate::error::Error> {
        match rich_text {
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
                            notionrs::object::color::Color::Blue => Some(String::from("#6987b8")),
                            notionrs::object::color::Color::Brown => Some(String::from("#8b4c3f")),
                            notionrs::object::color::Color::Gray => Some(String::from("#868e9c")),
                            notionrs::object::color::Color::Green => Some(String::from("#59b57c")),
                            notionrs::object::color::Color::Orange => Some(String::from("#bf7e71")),
                            notionrs::object::color::Color::Pink => Some(String::from("#c9699e")),
                            notionrs::object::color::Color::Purple => Some(String::from("#9771bd")),
                            notionrs::object::color::Color::Red => Some(String::from("#b36472")),
                            notionrs::object::color::Color::Yellow => Some(String::from("#b8a36e")),
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
                        href: text.link.clone().map(|l| l.url),
                        favicon: if let Some(l) = text.link {
                            self.fetch_page_meta(&l.url)
                                .await
                                .ok()
                                .and_then(|p| p.favicon)
                        } else {
                            None
                        },
                    },
                    slots: None,
                };

                Ok(component.into())
            }
            notionrs::object::rich_text::RichText::Mention {
                mention,
                annotations,
                plain_text,
                href,
            } => todo!(),
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
        }
    }

    async fn fetch_page_meta(&self, href: &str) -> Result<PageMeta, crate::error::Error> {
        let mut page_meta = PageMeta::default();

        let response = reqwest::Client::new()
            .get(href)
            .header("user-agent", "Rust - reqwest")
            .send()
            .await?
            .text()
            .await?;

        let document = scraper::Html::parse_document(&response);

        // title

        let title = document
            .select(&scraper::Selector::parse("title")?)
            .next()
            .map(|element| element.text().collect::<String>());

        let og_title_selector = scraper::Selector::parse("meta[property='og:title']")?;

        if let Some(element) = document.select(&og_title_selector).next() {
            if let Some(content) = element.value().attr("content") {
                page_meta.title = Some(content.to_string());
            }
        }

        if let Some(title) = title {
            page_meta.title = Some(title);
        }

        // description

        let description = document
            .select(&scraper::Selector::parse("meta[name='description']")?)
            .next()
            .map(|element| element.value().attr("content").unwrap().to_string());

        if let Some(description) = description {
            page_meta.description = Some(description);
        }

        let og_description_selector = scraper::Selector::parse("meta[property='og:description']")?;

        if let Some(element) = document.select(&og_description_selector).next() {
            if let Some(content) = element.value().attr("content") {
                page_meta.description = Some(content.to_string());
            }
        }

        // image

        let og_image_selector = scraper::Selector::parse("meta[property='og:image']")?;

        if let Some(element) = document.select(&og_image_selector).next() {
            if let Some(content) = element.value().attr("content") {
                page_meta.image = Some(content.to_string());
            }
        };

        // favicon

        let favicon_selector = scraper::Selector::parse(r#"link[rel="icon"]"#)?;

        page_meta.favicon = document
            .select(&favicon_selector)
            .next()
            .and_then(|f| f.value().attr("href").map(String::from))
            .and_then(|favicon_href| {
                if favicon_href.starts_with("http://") || favicon_href.starts_with("https://") {
                    Some(favicon_href)
                } else {
                    url::Url::parse(&href)
                        .and_then(|s| s.join(&favicon_href))
                        .map(|s| s.to_string())
                        .ok()
                }
            });

        Ok(page_meta)
    }
}
