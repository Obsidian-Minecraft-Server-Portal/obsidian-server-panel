use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Parses markdown content and converts it to styled text segments for rendering
pub fn parse_markdown_to_segments(markdown: &str) -> Vec<crate::TextSegment> {
    let mut segments = Vec::new();
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);

    // State tracking for nested styles
    let mut current_text = String::new();
    let mut heading_level = 0;
    let mut is_bold = false;
    let mut is_italic = false;
    let mut is_code = false;
    let mut is_link = false;
    let mut list_depth = 0;
    let mut list_item_prefix = String::new();

    // Theme colors (matching Slint theme)
    let text_primary = slint::Color::from_rgb_u8(240, 240, 240);
    let text_secondary = slint::Color::from_rgb_u8(180, 180, 180);
    let accent_color = slint::Color::from_rgb_u8(100, 149, 237);

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    heading_level = level as i32;
                }
                Tag::Strong => {
                    is_bold = true;
                }
                Tag::Emphasis => {
                    is_italic = true;
                }
                Tag::CodeBlock(_) => {
                    is_code = true;
                }
                Tag::Link { .. } => {
                    is_link = true;
                }
                Tag::List(_) => {
                    list_depth += 1;
                }
                Tag::Item => {
                    list_item_prefix = "  ".repeat((list_depth - 1) as usize) + "• ";
                    current_text.push_str(&list_item_prefix);
                }
                Tag::Paragraph => {
                    // Paragraphs are handled automatically
                }
                _ => {}
            },
            Event::End(tag_end) => {
                // Flush current text as a segment when tags end
                if !current_text.is_empty() {
                    let (font_size, font_weight, margin_top, margin_bottom) = match heading_level {
                        1 => (28.0, 900, 16.0, 12.0),
                        2 => (24.0, 700, 14.0, 10.0),
                        3 => (20.0, 700, 12.0, 8.0),
                        4 => (18.0, 600, 10.0, 6.0),
                        5 => (16.0, 600, 8.0, 6.0),
                        6 => (14.0, 600, 6.0, 4.0),
                        _ => {
                            if is_code {
                                (12.0, 400, 4.0, 4.0)
                            } else if is_bold {
                                (13.0, 700, 2.0, 2.0)
                            } else {
                                (13.0, 400, 2.0, 2.0)
                            }
                        }
                    };

                    let color = if heading_level > 0 {
                        text_primary.into()
                    } else if is_code {
                        accent_color.into()
                    } else {
                        text_secondary.into()
                    };

                    segments.push(crate::TextSegment {
                        text: current_text.clone().into(),
                        font_size,
                        font_weight,
                        color,
                        is_italic,
                        is_code,
                        is_link,
                        margin_top,
                        margin_bottom,
                    });

                    current_text.clear();
                }

                // Reset state based on ended tag
                match tag_end {
                    TagEnd::Heading(_) => {
                        heading_level = 0;
                    }
                    TagEnd::Strong => {
                        is_bold = false;
                    }
                    TagEnd::Emphasis => {
                        is_italic = false;
                    }
                    TagEnd::CodeBlock => {
                        is_code = false;
                    }
                    TagEnd::Link => {
                        is_link = false;
                    }
                    TagEnd::List(_) => {
                        list_depth -= 1;
                    }
                    TagEnd::Item => {
                        list_item_prefix.clear();
                    }
                    TagEnd::Paragraph => {
                        // Add spacing after paragraphs if not already handled
                        if !segments.is_empty() && !current_text.is_empty() {
                            current_text.push('\n');
                        }
                    }
                    _ => {}
                }
            }
            Event::Text(text) => {
                current_text.push_str(&text);
            }
            Event::Code(code) => {
                // Inline code - just append with backticks
                current_text.push('`');
                current_text.push_str(&code);
                current_text.push('`');
            }
            Event::SoftBreak => {
                current_text.push(' ');
            }
            Event::HardBreak => {
                current_text.push('\n');
            }
            Event::Rule => {
                segments.push(crate::TextSegment {
                    text: "─────────────────────────────────────────────────".into(),
                    font_size: 13.0,
                    font_weight: 400,
                    color: text_secondary.into(),
                    is_italic: false,
                    is_code: false,
                    is_link: false,
                    margin_top: 8.0,
                    margin_bottom: 8.0,
                });
            }
            _ => {}
        }
    }

    // Flush any remaining text
    if !current_text.is_empty() {
        segments.push(crate::TextSegment {
            text: current_text.into(),
            font_size: 13.0,
            font_weight: 400,
            color: text_secondary.into(),
            is_italic,
            is_code,
            is_link,
            margin_top: 2.0,
            margin_bottom: 2.0,
        });
    }

    segments
}
