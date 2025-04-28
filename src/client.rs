#[derive(Debug)]
pub struct Client {
    pub notionrs_client: notionrs::client::Client,
    pub reqwest_client: reqwest::Client,

    /// If true, unsupported blocks will be rendered as `Unsupported` blocks.
    /// If false, unsupported blocks will be skipped.
    pub enable_unsupported_block: bool,
}

impl Client {
    fn create_unsupported_component(&self, bloack_name: &str) -> jarkup_rs::Component {
        jarkup_rs::Unsupported {
            props: Some(jarkup_rs::UnsupportedProps {
                details: format!("Notion: `{} Block` is not supported.", bloack_name),
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

        let blocks = self
            .notionrs_client
            .get_block_children_all()
            .block_id(block_id)
            .send()
            .await?;

        for block in blocks {
            match block.block {
                notionrs::object::block::Block::Audio { audio: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Audio"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::Bookmark { bookmark } => {
                    let html = self
                        .reqwest_client
                        .get(&bookmark.url)
                        .send()
                        .await?
                        .text()
                        .await?;

                    let meta_scraper = html_meta_scraper::MetaScraper::new(&html);

                    let title = meta_scraper.title();
                    let description = meta_scraper.description();
                    let image = meta_scraper.image();

                    let component = jarkup_rs::Bookmark {
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
                notionrs::object::block::Block::Breadcrumb { breadcrumb: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Breadcrumb"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::BulletedListItem { bulleted_list_item } => {
                    let list_item_component = jarkup_rs::ListItem {
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
                        props: Some(jarkup_rs::ListProps {
                            list_style: Some(jarkup_rs::ListStyle::Unordered),
                        }),
                        slots: jarkup_rs::ListSlots {
                            default: vec![list_item_component.into()],
                        },
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Callout { callout } => {
                    let maybe_paragraph_component: Option<jarkup_rs::Component> =
                        if callout.rich_text.len() > 0 {
                            Some(
                                jarkup_rs::Paragraph {
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
                        props: Some(jarkup_rs::CalloutProps {
                            r#type: Some(match callout.color {
                                notionrs::object::color::Color::Default
                                | notionrs::object::color::Color::DefaultBackground
                                | notionrs::object::color::Color::Blue
                                | notionrs::object::color::Color::BlueBackground
                                | notionrs::object::color::Color::Gray
                                | notionrs::object::color::Color::GrayBackground => {
                                    jarkup_rs::CalloutType::Note
                                }
                                notionrs::object::color::Color::Green
                                | notionrs::object::color::Color::GreenBackground => {
                                    jarkup_rs::CalloutType::Tip
                                }
                                notionrs::object::color::Color::Purple
                                | notionrs::object::color::Color::PurpleBackground => {
                                    jarkup_rs::CalloutType::Important
                                }
                                notionrs::object::color::Color::Yellow
                                | notionrs::object::color::Color::YellowBackground
                                | notionrs::object::color::Color::Orange
                                | notionrs::object::color::Color::OrangeBackground
                                | notionrs::object::color::Color::Brown
                                | notionrs::object::color::Color::BrownBackground => {
                                    jarkup_rs::CalloutType::Warning
                                }
                                notionrs::object::color::Color::Red
                                | notionrs::object::color::Color::RedBackground
                                | notionrs::object::color::Color::Pink
                                | notionrs::object::color::Color::PinkBackground => {
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
                notionrs::object::block::Block::ChildDatabase { child_database: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("ChildDatabase"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::ChildPage { child_page: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("ChildPage"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::Code { code } => {
                    let component = jarkup_rs::CodeBlock {
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
                        slots: Some(jarkup_rs::CodeBlockSlots {
                            default: self.convert_rich_text(code.caption).await?,
                        }),
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::ColumnList { column_list: _ } => continue,
                notionrs::object::block::Block::Column { column: _ } => continue,
                notionrs::object::block::Block::Divider { divider: _ } => {
                    let component = jarkup_rs::Divider {
                        props: None,
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Embed { embed: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Embed"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::Equation { equation } => {
                    let component = jarkup_rs::Katex {
                        props: jarkup_rs::KatexProps {
                            expression: equation.expression,
                        },
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::File { file } => {
                    let component = jarkup_rs::File {
                        props: jarkup_rs::FileProps {
                            src: file.get_url(),
                            name: match file {
                                notionrs::object::file::File::External(external_file) => {
                                    external_file.name
                                }
                                notionrs::object::file::File::Uploaded(uploaded_file) => {
                                    uploaded_file.name
                                }
                            },
                        },
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Heading1 { heading_1 } => {
                    let component = jarkup_rs::Heading {
                        props: jarkup_rs::HeadingProps {
                            level: jarkup_rs::HeadingLevel::H1,
                        },
                        slots: jarkup_rs::HeadingSlots {
                            default: self.convert_rich_text(heading_1.rich_text).await?,
                        },
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Heading2 { heading_2 } => {
                    let component = jarkup_rs::Heading {
                        props: jarkup_rs::HeadingProps {
                            level: jarkup_rs::HeadingLevel::H2,
                        },
                        slots: jarkup_rs::HeadingSlots {
                            default: self.convert_rich_text(heading_2.rich_text).await?,
                        },
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Heading3 { heading_3 } => {
                    let component = jarkup_rs::Heading {
                        props: jarkup_rs::HeadingProps {
                            level: jarkup_rs::HeadingLevel::H3,
                        },
                        slots: jarkup_rs::HeadingSlots {
                            default: self.convert_rich_text(heading_3.rich_text).await?,
                        },
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Image { image } => {
                    let maybe_caption = match image.clone() {
                        notionrs::object::file::File::External(external_file) => {
                            external_file.caption
                        }
                        notionrs::object::file::File::Uploaded(uploaded_file) => {
                            uploaded_file.caption
                        }
                    }
                    .map(|c| c.into_iter().map(|r| r.to_string()).collect::<String>());

                    let component = jarkup_rs::Image {
                        props: jarkup_rs::ImageProps {
                            src: image.get_url(),
                            alt: maybe_caption,
                        },
                        slots: None,
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::LinkPreview { link_preview: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("LinkPreview"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::NumberedListItem { numbered_list_item } => {
                    let list_item_component = jarkup_rs::ListItem {
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
                        props: Some(jarkup_rs::ListProps {
                            list_style: Some(jarkup_rs::ListStyle::Ordered),
                        }),
                        slots: jarkup_rs::ListSlots {
                            default: vec![list_item_component.into()],
                        },
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Paragraph { paragraph } => {
                    let component = jarkup_rs::Paragraph {
                        props: None,
                        slots: jarkup_rs::ParagraphSlots {
                            default: self.convert_rich_text(paragraph.rich_text).await?,
                        },
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Pdf { pdf: _ } => {}
                notionrs::object::block::Block::Quote { quote } => {
                    let maybe_paragraph_component: Option<jarkup_rs::Component> =
                        if quote.rich_text.len() > 0 {
                            let paragraph = jarkup_rs::Paragraph {
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
                        props: None,
                        slots: jarkup_rs::BlockQuoteSlots {
                            default: merged_components,
                        },
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::SyncedBlock { synced_block: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("SyncedBlock"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::TableOfContents {
                    table_of_contents: _,
                } => continue,
                notionrs::object::block::Block::Table { table } => {
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
                notionrs::object::block::Block::TableRow { table_row } => {
                    let mut cell_components: Vec<jarkup_rs::Component> = Vec::new();

                    for cell in table_row.cells {
                        let children_inline_componense = self.convert_rich_text(cell).await?;

                        let component = jarkup_rs::TableCell {
                            props: None,
                            slots: jarkup_rs::TableCellSlots {
                                default: children_inline_componense,
                            },
                        };

                        cell_components.push(component.into());
                    }

                    let row_component = jarkup_rs::TableRow {
                        props: None,
                        slots: jarkup_rs::TableRowSlots {
                            default: cell_components,
                        },
                    };

                    components.push(row_component.into());
                }
                notionrs::object::block::Block::Template { template: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Template"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::ToDo { to_do: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("ToDo"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::Toggle { toggle } => {
                    let children_components = if block.has_children {
                        self.convert_block(&block.id).await?
                    } else {
                        vec![]
                    };

                    let summary_components = self.convert_rich_text(toggle.rich_text).await?;

                    let component = jarkup_rs::Toggle {
                        props: None,
                        slots: jarkup_rs::ToggleSlots {
                            default: children_components,
                            summary: summary_components,
                        },
                    };

                    components.push(component.into());
                }
                notionrs::object::block::Block::Video { video: _ } => {
                    if self.enable_unsupported_block {
                        components.push(self.create_unsupported_component("Video"));
                    } else {
                        continue;
                    }
                }
                notionrs::object::block::Block::Unsupported => {
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
                    href: _,
                } => {
                    let component = jarkup_rs::Text {
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
        let res = self.reqwest_client.get(url).send().await.ok()?;
        let html = res.text().await.ok()?;
        let meta_scraper = html_meta_scraper::MetaScraper::new(&html);
        meta_scraper.favicon()
    }
}
