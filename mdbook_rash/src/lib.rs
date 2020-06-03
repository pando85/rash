use mdbook::book::{Book, BookItem, Chapter};
use mdbook::errors::Error;
use mdbook::preprocess::PreprocessorContext;
use regex::{Match, Regex};

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub static SUPPORTED_RENDERER: &[&str] = &["html"];

lazy_static! {
    static ref RE: Regex = Regex::new(
        r"(?x)                                     # insignificant whitespace mode
        \\\{\{\#.*\}\}                             # match escaped link
        |                                          # or
        \{\{\s*                                    # link opening parens and whitespace
        \#([a-zA-Z0-9_]+)                          # link type
        \s+                                        # separating whitespace
        ([a-zA-Z0-9\s_.,\[\]\(\)\|'\-\\/`\#:/\\]+) # all doc
        \s*\}\}                                    # whitespace and link closing parens"
    )
    .unwrap();
}

fn get_matches<'a>(ch: &'a Chapter) -> Option<Vec<(Match<'a>, String)>> {
    RE.captures_iter(&ch.content)
        .map(|cap| match (cap.get(0), cap.get(1), cap.get(2)) {
            (Some(origin), Some(typ), Some(rest)) => match (typ.as_str(), rest.as_str()) {
                ("include_module", content) => {
                    Some((origin, content.replace("/// ", "").replace("///", "")))
                }
                _ => None,
            },
            _ => None,
        })
        .collect::<Option<Vec<(Match, String)>>>()
}

fn replace_matches(captures: Vec<(Match, String)>, ch: &mut Chapter) {
    for capture in captures.iter() {
        let new_content = capture.1.clone();
        let name = new_content.split('\n').next().unwrap().replace("# ", "");
        let mut new_ch = Chapter::new(
            &name,
            new_content,
            format!("{}.md", &name),
            vec![ch.name.clone()],
        );

        let mut new_section_number = ch.number.clone().unwrap();
        new_section_number.push((ch.sub_items.len() + 1) as u32);
        new_ch.number = Some(new_section_number);
        ch.sub_items.push(BookItem::Chapter(new_ch));

        ch.content = RE.replace(&ch.content, "").to_string();
        info!("module {} added", &name);
    }
}

pub fn run(_ctx: &PreprocessorContext, book: Book) -> Result<Book, Error> {
    let mut new_book = book;

    new_book.for_each_mut(|section: &mut BookItem| {
        if let BookItem::Chapter(ref mut ch) = *section {
            let ch_copy = ch.clone();
            if let Some(captures) = get_matches(&ch_copy) {
                replace_matches(captures, ch);
            };
        }
    });

    Ok(new_book)
}