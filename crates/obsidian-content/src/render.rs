use pulldown_cmark::{Options, Parser, html};

pub fn markdown(source: &str) -> String {
	let options = Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
	let mut rendered = String::new();
	html::push_html(&mut rendered, Parser::new_ext(source, options));
	html(&rendered)
}

pub fn html(source: &str) -> String {
	let mut builder = ammonia::Builder::default();
	builder.link_rel(Some("noopener noreferrer nofollow"));
	builder.clean(source).to_string()
}
