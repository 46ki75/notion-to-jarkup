use futures::TryStreamExt;
use notionrs::PaginateExt;
use notionrs_types::prelude::*;

#[derive(Debug)]
pub struct Client {
    pub notionrs_client: notionrs::client::Client,
    pub reqwest_client: reqwest::Client,

    /// If true, unsupported blocks will be rendered as `Unsupported` blocks.
    /// If false, unsupported blocks will be skipped.
    pub enable_unsupported_block: bool,
}

impl Client {
    fn create_unsupported_component(&self, block_name: &str) -> jarkup_rs::Component {
        jarkup_rs::Unsupported {
            id: None,
            props: Some(jarkup_rs::UnsupportedProps {
                details: format!("Notion: `{} Block` is not supported.", block_name),
            }),
            slots: None,
        }
        .into()
    }

    #[async_recursion::async_recursion]
    pub async fn convert_block(
        &self,
        block_id: &str,
    ) -> Result<Vec<jarkup_rs::Component>, crate::error::Error> {
        let mut components: Vec<jarkup_rs::Component> = Vec::new();

        let blocks: Vec<BlockResponse> = self
            .notionrs_client
            .get_block_children()
            .block_id(block_id)
            .into_stream()
            .try_collect()
            .await?;

        for block in blocks {
            match block.block {
                notionrs_types::object::block::Block::Audio { audio: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Audio"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::Bookmark { bookmark } => {
                    let html = self
                        .reqwest_client
                        .get(&bookmark.url)
                        .header("user-agent", "notion-to-jarkup")
                        .send()
                        .await?
                        .text()
                        .await?;

                    let meta_scraper = html_meta_scraper::MetaScraper::new(&html);

                    let title = meta_scraper.title();
                    let description = meta_scraper.description();
                    let image = meta_scraper.image();

                    let component = jarkup_rs::Bookmark {
                        id: Some(block.id),
                        props: jarkup_rs::BookmarkProps {
                            url: bookmark.url,
                            title,
                            description,
                            image,
                        },
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::Breadcrumb { breadcrumb: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Breadcrumb"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::BulletedListItem { bulleted_list_item } => {
                    let list_item_component = jarkup_rs::ListItem {
                        id: Some(block.id),
                        props: None,
                        slots: jarkup_rs::ListItemSlots {
                            default: self.convert_rich_text(bulleted_list_item.rich_text).await?,
                        },
                    };

                    let maybe_prev_component = components.last_mut().and_then(|c| match c {
                        jarkup_rs::Component::InlineComponent(_) => None,
                        jarkup_rs::Component::BlockComponent(block_component) => {
                            match block_component {
                                jarkup_rs::BlockComponent::List(list) => Some(list),
                                _ => None,
                            }
                        }
                    });

                    match maybe_prev_component {
                        Some(prev_component) => {
                            let is_unordered = prev_component
                                .props
                                .clone()
                                .map(|p| {
                                    matches!(p.list_style, Some(jarkup_rs::ListStyle::Unordered))
                                })
                                .unwrap_or(true);

                            if is_unordered {
                                prev_component
                                    .slots
                                    .default
                                    .push(list_item_component.into());

                                continue;
                            }
                        }

                        None => {}
                    };

                    let component = jarkup_rs::List {
                        id: None,
                        props: Some(jarkup_rs::ListProps {
                            list_style: Some(jarkup_rs::ListStyle::Unordered),
                        }),
                        slots: jarkup_rs::ListSlots {
                            default: vec![list_item_component.into()],
                        },
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::Callout { callout } => {
                    let maybe_paragraph_component: Option<jarkup_rs::Component> =
                        if callout.rich_text.len() > 0 {
                            Some(
                                jarkup_rs::Paragraph {
                                    id: Some(block.id.clone()),
                                    props: None,
                                    slots: jarkup_rs::ParagraphSlots {
                                        default: self.convert_rich_text(callout.rich_text).await?,
                                    },
                                }
                                .into(),
                            )
                        } else {
                            None
                        };

                    let maybe_children_components = if block.has_children {
                        Some(self.convert_block(&block.id).await?)
                    } else {
                        None
                    };

                    let merged_children_components = maybe_paragraph_component
                        .into_iter()
                        .chain(maybe_children_components.unwrap_or_default())
                        .collect::<Vec<jarkup_rs::Component>>();

                    let component = jarkup_rs::Callout {
                        id: Some(block.id),
                        props: Some(jarkup_rs::CalloutProps {
                            r#type: Some(match callout.color {
                                notionrs_types::object::color::Color::Default
                                | notionrs_types::object::color::Color::DefaultBackground
                                | notionrs_types::object::color::Color::Blue
                                | notionrs_types::object::color::Color::BlueBackground
                                | notionrs_types::object::color::Color::Gray
                                | notionrs_types::object::color::Color::GrayBackground => {
                                    jarkup_rs::CalloutType::Note
                                }
                                notionrs_types::object::color::Color::Green
                                | notionrs_types::object::color::Color::GreenBackground => {
                                    jarkup_rs::CalloutType::Tip
                                }
                                notionrs_types::object::color::Color::Purple
                                | notionrs_types::object::color::Color::PurpleBackground => {
                                    jarkup_rs::CalloutType::Important
                                }
                                notionrs_types::object::color::Color::Yellow
                                | notionrs_types::object::color::Color::YellowBackground
                                | notionrs_types::object::color::Color::Orange
                                | notionrs_types::object::color::Color::OrangeBackground
                                | notionrs_types::object::color::Color::Brown
                                | notionrs_types::object::color::Color::BrownBackground => {
                                    jarkup_rs::CalloutType::Warning
                                }
                                notionrs_types::object::color::Color::Red
                                | notionrs_types::object::color::Color::RedBackground
                                | notionrs_types::object::color::Color::Pink
                                | notionrs_types::object::color::Color::PinkBackground => {
                                    jarkup_rs::CalloutType::Caution
                                }
                            }),
                        }),
                        slots: jarkup_rs::CalloutSlots {
                            default: merged_children_components,
                        },
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::ChildDatabase { child_database: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("ChildDatabase"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::ChildPage { child_page: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("ChildPage"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::Code { code } => {
                    let component: jarkup_rs::Component = match code.language {
                        Language::Mermaid => jarkup_rs::Mermaid {
                            id: Some(block.id),
                            props: jarkup_rs::MermaidProps {
                                code: code
                                    .rich_text
                                    .clone()
                                    .into_iter()
                                    .map(|r| r.to_string())
                                    .collect::<Vec<String>>()
                                    .join(""),
                            },
                            slots: None,
                        }
                        .into(),
                        _ => jarkup_rs::CodeBlock {
                            id: Some(block.id),
                            props: jarkup_rs::CodeBlockProps {
                                code: code
                                    .rich_text
                                    .clone()
                                    .into_iter()
                                    .map(|r| r.to_string())
                                    .collect::<Vec<String>>()
                                    .join(""),
                                language: code.language.to_string(),
                            },
                            slots: if code.caption.len() > 0 {
                                Some(jarkup_rs::CodeBlockSlots {
                                    default: self.convert_rich_text(code.caption).await?,
                                })
                            } else {
                                None
                            },
                        }
                        .into(),
                    };

                    components.push(component);
                }
                notionrs_types::object::block::Block::ColumnList { column_list: _ } => continue,
                notionrs_types::object::block::Block::Column { column: _ } => continue,
                notionrs_types::object::block::Block::Divider { divider: _ } => {
                    let component = jarkup_rs::Divider {
                        id: Some(block.id),
                        props: None,
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::Embed { embed: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Embed"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::Equation { equation } => {
                    let component = jarkup_rs::Katex {
                        id: Some(block.id),
                        props: jarkup_rs::KatexProps {
                            expression: equation.expression,
                        },
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::File { file } => {
                    let component = jarkup_rs::File {
                        id: Some(block.id),
                        props: jarkup_rs::FileProps {
                            src: file.get_url(),
                            name: match file {
                                notionrs_types::object::file::File::External(external_file) => {
                                    external_file.name
                                }
                                notionrs_types::object::file::File::NotionHosted(
                                    notion_hosted_file,
                                ) => notion_hosted_file.name,
                                _ => Some(String::from("untitled")),
                            },
                        },
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::Heading1 { heading_1 } => {
                    let component = self
                        .convert_heading_block(heading_1, &block.id, jarkup_rs::HeadingLevel::H1)
                        .await?;

                    if let Some(c) = component {
                        components.push(c);
                    } else {
                        continue;
                    };
                }
                notionrs_types::object::block::Block::Heading2 { heading_2 } => {
                    let component = self
                        .convert_heading_block(heading_2, &block.id, jarkup_rs::HeadingLevel::H2)
                        .await?;

                    if let Some(c) = component {
                        components.push(c);
                    } else {
                        continue;
                    };
                }
                notionrs_types::object::block::Block::Heading3 { heading_3 } => {
                    let component = self
                        .convert_heading_block(heading_3, &block.id, jarkup_rs::HeadingLevel::H3)
                        .await?;

                    if let Some(c) = component {
                        components.push(c);
                    } else {
                        continue;
                    };
                }
                notionrs_types::object::block::Block::Image { image } => {
                    let maybe_caption = match image.clone() {
                        notionrs_types::object::file::File::External(external_file) => {
                            external_file
                                .caption
                                .map(|c| c.into_iter().map(|c| c.to_string()).collect::<String>())
                        }
                        notionrs_types::object::file::File::NotionHosted(notion_hosted_file) => {
                            notion_hosted_file
                                .caption
                                .map(|c| c.into_iter().map(|c| c.to_string()).collect::<String>())
                        }
                        _ => Some(String::from("untitled")),
                    };

                    let component = jarkup_rs::Image {
                        id: Some(block.id),
                        props: jarkup_rs::ImageProps {
                            src: image.get_url(),
                            alt: maybe_caption,
                        },
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::LinkPreview { link_preview: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("LinkPreview"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::NumberedListItem { numbered_list_item } => {
                    let list_item_component = jarkup_rs::ListItem {
                        id: Some(block.id),
                        props: None,
                        slots: jarkup_rs::ListItemSlots {
                            default: self.convert_rich_text(numbered_list_item.rich_text).await?,
                        },
                    };

                    let maybe_prev_component = components.last_mut().and_then(|c| match c {
                        jarkup_rs::Component::InlineComponent(_) => None,
                        jarkup_rs::Component::BlockComponent(block_component) => {
                            match block_component {
                                jarkup_rs::BlockComponent::List(list) => Some(list),
                                _ => None,
                            }
                        }
                    });

                    match maybe_prev_component {
                        Some(prev_component) => {
                            let is_ordered = prev_component
                                .props
                                .clone()
                                .map(|p| {
                                    matches!(p.list_style, Some(jarkup_rs::ListStyle::Ordered))
                                })
                                .unwrap_or(true);

                            if is_ordered {
                                prev_component
                                    .slots
                                    .default
                                    .push(list_item_component.into());

                                continue;
                            }
                        }

                        None => {}
                    };

                    let component = jarkup_rs::List {
                        id: None,
                        props: Some(jarkup_rs::ListProps {
                            list_style: Some(jarkup_rs::ListStyle::Ordered),
                        }),
                        slots: jarkup_rs::ListSlots {
                            default: vec![list_item_component.into()],
                        },
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::Paragraph { paragraph } => {
                    let component = jarkup_rs::Paragraph {
                        id: Some(block.id),
                        props: None,
                        slots: jarkup_rs::ParagraphSlots {
                            default: self.convert_rich_text(paragraph.rich_text).await?,
                        },
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::Pdf { pdf: _ } => {}
                notionrs_types::object::block::Block::Quote { quote } => {
                    let maybe_paragraph_component: Option<jarkup_rs::Component> =
                        if quote.rich_text.len() > 0 {
                            let paragraph = jarkup_rs::Paragraph {
                                id: Some(block.id.clone()),
                                props: None,
                                slots: jarkup_rs::ParagraphSlots {
                                    default: self.convert_rich_text(quote.rich_text).await?,
                                },
                            };
                            Some(paragraph.into())
                        } else {
                            None
                        };

                    let maybe_children_components = if block.has_children {
                        self.convert_block(&block.id).await?
                    } else {
                        vec![]
                    };

                    let merged_components = maybe_paragraph_component
                        .into_iter()
                        .chain(maybe_children_components)
                        .collect::<Vec<jarkup_rs::Component>>();

                    let component = jarkup_rs::BlockQuote {
                        id: Some(block.id),
                        props: None,
                        slots: jarkup_rs::BlockQuoteSlots {
                            default: merged_components,
                        },
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::SyncedBlock { synced_block: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("SyncedBlock"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::TableOfContents {
                    table_of_contents: _,
                } => continue,
                notionrs_types::object::block::Block::Table { table } => {
                    let mut all_children_rows = self.convert_block(&block.id).await?;

                    let maybe_header_row =
                        if table.has_column_header && all_children_rows.len() > 0 {
                            let first = all_children_rows.remove(0);
                            let maybe_row_components = if let jarkup_rs::Component::BlockComponent(
                                block_component,
                            ) = first
                            {
                                if let jarkup_rs::BlockComponent::TableRow(table_row) =
                                    block_component
                                {
                                    Some(table_row.to_owned().into())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            Some(maybe_row_components)
                        } else {
                            None
                        }
                        .flatten()
                        .map(|table_row| vec![table_row]);

                    let body_rows = all_children_rows
                        .into_iter()
                        .filter_map(|row| {
                            let maybe_row_component = if let jarkup_rs::Component::BlockComponent(
                                block_component,
                            ) = row
                            {
                                if let jarkup_rs::BlockComponent::TableRow(table_row) =
                                    block_component
                                {
                                    Some(table_row.to_owned().into())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            maybe_row_component
                        })
                        .collect::<Vec<jarkup_rs::Component>>();

                    let component = jarkup_rs::Table {
                        id: Some(block.id),
                        props: Some(jarkup_rs::TableProps {
                            has_column_header: Some(table.has_column_header),
                            has_row_header: Some(table.has_row_header),
                            caption: None,
                        }),
                        slots: jarkup_rs::TableSlots {
                            header: maybe_header_row,
                            body: body_rows,
                        },
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::TableRow { table_row } => {
                    let mut cell_components: Vec<jarkup_rs::Component> = Vec::new();

                    for cell in table_row.cells {
                        let children_inline_componense = self.convert_rich_text(cell).await?;

                        let component = jarkup_rs::TableCell {
                            id: None,
                            props: None,
                            slots: jarkup_rs::TableCellSlots {
                                default: children_inline_componense,
                            },
                        };

                        cell_components.push(component.into());
                    }

                    let row_component = jarkup_rs::TableRow {
                        id: Some(block.id),
                        props: None,
                        slots: jarkup_rs::TableRowSlots {
                            default: cell_components,
                        },
                    };

                    components.push(row_component.into());
                }
                notionrs_types::object::block::Block::Template { template: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Template"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::ToDo { to_do: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("ToDo"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::Toggle { toggle } => {
                    let children_components = if block.has_children {
                        self.convert_block(&block.id).await?
                    } else {
                        vec![]
                    };

                    let summary_components = self.convert_rich_text(toggle.rich_text).await?;

                    let component = jarkup_rs::Toggle {
                        id: Some(block.id),
                        props: None,
                        slots: jarkup_rs::ToggleSlots {
                            default: children_components,
                            summary: summary_components,
                        },
                    };

                    components.push(component.into());
                }
                notionrs_types::object::block::Block::Video { video: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Video"));
                    } else {
                        continue;
                    }
                }
                notionrs_types::object::block::Block::Unsupported => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Unsupported"));
                    } else {
                        continue;
                    }
                }
            }
        }

        Ok(components)
    }

    pub async fn convert_rich_text(
        &self,
        rich_text_vec: Vec<RichText>,
    ) -> Result<Vec<jarkup_rs::InlineComponent>, crate::error::Error> {
        let mut components: Vec<jarkup_rs::InlineComponent> = Vec::new();

        for rich_text in rich_text_vec {
            let component: Result<jarkup_rs::InlineComponent, crate::error::Error> = match rich_text
            {
                RichText::Text {
                    text,
                    annotations,
                    plain_text,
                    href: _,
                } => {
                    let component = if self.is_kbd(&plain_text, annotations.code) {
                        jarkup_rs::Text {
                            props: jarkup_rs::TextProps {
                                text: plain_text,
                                kbd: Some(true),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    } else {
                        jarkup_rs::Text {
                            id: None,
                            props: jarkup_rs::TextProps {
                                text: plain_text,
                                color: match annotations.color {
                                    notionrs_types::object::color::Color::Default => None,
                                    notionrs_types::object::color::Color::Blue => {
                                        Some(String::from("#6987b8"))
                                    }
                                    notionrs_types::object::color::Color::Brown => {
                                        Some(String::from("#8b4c3f"))
                                    }
                                    notionrs_types::object::color::Color::Gray => {
                                        Some(String::from("#868e9c"))
                                    }
                                    notionrs_types::object::color::Color::Green => {
                                        Some(String::from("#59b57c"))
                                    }
                                    notionrs_types::object::color::Color::Orange => {
                                        Some(String::from("#bf7e71"))
                                    }
                                    notionrs_types::object::color::Color::Pink => {
                                        Some(String::from("#c9699e"))
                                    }
                                    notionrs_types::object::color::Color::Purple => {
                                        Some(String::from("#9771bd"))
                                    }
                                    notionrs_types::object::color::Color::Red => {
                                        Some(String::from("#b36472"))
                                    }
                                    notionrs_types::object::color::Color::Yellow => {
                                        Some(String::from("#b8a36e"))
                                    }
                                    _ => None,
                                },
                                background_color: match annotations.color {
                                    notionrs_types::object::color::Color::Default => None,
                                    notionrs_types::object::color::Color::BlueBackground => {
                                        Some(String::from("#6987b8"))
                                    }
                                    notionrs_types::object::color::Color::BrownBackground => {
                                        Some(String::from("#8b4c3f"))
                                    }
                                    notionrs_types::object::color::Color::GrayBackground => {
                                        Some(String::from("#868e9c"))
                                    }
                                    notionrs_types::object::color::Color::GreenBackground => {
                                        Some(String::from("#59b57c"))
                                    }
                                    notionrs_types::object::color::Color::OrangeBackground => {
                                        Some(String::from("#bf7e71"))
                                    }
                                    notionrs_types::object::color::Color::PinkBackground => {
                                        Some(String::from("#c9699e"))
                                    }
                                    notionrs_types::object::color::Color::PurpleBackground => {
                                        Some(String::from("#9771bd"))
                                    }
                                    notionrs_types::object::color::Color::RedBackground => {
                                        Some(String::from("#b36472"))
                                    }
                                    notionrs_types::object::color::Color::YellowBackground => {
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
                                kbd: None,
                                ruby: None,
                                favicon: if let Some(l) = &text.link {
                                    self.fetch_favicon_by_url(&l.url).await
                                } else {
                                    None
                                },
                                href: text.link.map(|l| l.url),
                            },
                            slots: None,
                        }
                    };

                    Ok(component.into())
                }
                RichText::Mention {
                    mention,
                    annotations: _annotations,
                    plain_text,
                    href: _href,
                } => {
                    let component: Result<jarkup_rs::InlineComponent, crate::error::Error> =
                        match mention {
                            Mention::User { user: _ } => {
                                continue;
                            }
                            Mention::Date { date: _ } => {
                                continue;
                            }
                            Mention::LinkPreview { link_preview: _ } => {
                                continue;
                            }
                            Mention::LinkMention { link_mention } => {
                                let component = jarkup_rs::Text {
                                    id: None,
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
                            Mention::TemplateMention {
                                template_mention: _,
                            } => {
                                continue;
                            }
                            Mention::Page { page: _ } => {
                                continue;
                            }
                            Mention::Database { database: _ } => {
                                continue;
                            }
                            Mention::CustomEmoji { custom_emoji } => {
                                let component = jarkup_rs::Icon {
                                    id: Some(custom_emoji.id),
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
                RichText::Equation {
                    equation,
                    annotations: _annotations,
                    plain_text: _plain_text,
                    href: _href,
                } => {
                    let component = jarkup_rs::Text {
                        id: None,
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

            components.push(component?);
        }

        return Ok(components);
    }

    pub(crate) async fn fetch_favicon_by_url(&self, url: &str) -> Option<String> {
        let res = self
            .reqwest_client
            .get(url)
            .header("user-agent", "notion-to-jarkup")
            .send()
            .await
            .ok()?;
        let html = res.text().await.ok()?;
        let parsed_url = url::Url::parse(url).ok()?;
        let scheme = parsed_url.scheme();
        let host = if let url::Host::Domain(domain) = parsed_url.host()? {
            Some(domain.to_string())
        } else {
            None
        }?;
        let meta_scraper = html_meta_scraper::MetaScraper::new(&html);
        let base_url = url::Url::parse(&format!("{scheme}://{host}",)).ok()?;
        let favicon_url = base_url.join(&meta_scraper.favicon()?).ok()?.to_string();
        Some(favicon_url)
    }

    pub(crate) async fn convert_heading_block(
        &self,
        heading_block: HeadingBlock,
        block_id: &str,
        level: jarkup_rs::HeadingLevel,
    ) -> Result<Option<jarkup_rs::Component>, crate::error::Error> {
        let component: jarkup_rs::Component = if heading_block.is_toggleable {
            let children = self.convert_block(block_id).await?;

            jarkup_rs::Toggle {
                id: Some(block_id.to_owned()),
                props: None,
                slots: jarkup_rs::ToggleSlots {
                    default: children,
                    summary: self.convert_rich_text(heading_block.rich_text).await?,
                },
            }
            .into()
        } else {
            jarkup_rs::Heading {
                id: Some(block_id.to_owned()),
                props: jarkup_rs::HeadingProps { level },
                slots: jarkup_rs::HeadingSlots {
                    default: self.convert_rich_text(heading_block.rich_text).await?,
                },
            }
            .into()
        };

        Ok(Some(component))
    }

    pub(crate) fn is_kbd(&self, plain_text: &str, is_code: bool) -> bool {
        const SPECIAL_KEYS: &[&str] = &[
            "ctrl",
            "shift",
            "alt",
            "meta",
            "escape",
            "tab",
            "capslock",
            "enter",
            "backspace",
            "space",
            "arrowup",
            "arrowdown",
            "arrowleft",
            "arrowright",
            "insert",
            "delete",
            "home",
            "end",
            "pageup",
            "pagedown",
            "f1",
            "f2",
            "f3",
            "f4",
            "f5",
            "f6",
            "f7",
            "f8",
            "f9",
            "f10",
            "f11",
            "f12",
            "contextmenu",
            "numlock",
            "scrolllock",
            "pause",
        ];

        if !is_code {
            return false;
        };

        if plain_text.len() == 1 {
            return true;
        };

        if SPECIAL_KEYS.contains(&plain_text.to_lowercase().as_ref()) {
            return true;
        };

        return false;
    }
}
