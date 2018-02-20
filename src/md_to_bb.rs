
use regex::{Regex, RegexBuilder};

lazy_static! {
	static ref LINK:   Regex = Regex::new(r#"\[(.*?)\]\((.*?)\)"#).unwrap();
	static ref CODE3:  Regex = RegexBuilder::new(r#"```(.*?)```"#).multi_line(true).dot_matches_new_line(true).build().unwrap();
	static ref CODE2:  Regex = RegexBuilder::new(r#"``(.*?)``"#  ).multi_line(true).dot_matches_new_line(true).build().unwrap();
	static ref CODE1:  Regex = Regex::new(r#"`(.*?)`"#           ).unwrap();
	static ref H2:     Regex = Regex::new(r#"(^|\n)##(.*)"#      ).unwrap();
	static ref H1:     Regex = Regex::new(r#"(^|\n)#(.*)"#       ).unwrap();
	static ref BOLD:   Regex = Regex::new(r#"\*\*(.*?)\*\*"#     ).unwrap();
	static ref ITALIC: Regex = Regex::new(r#"\*(.*?)\*"#         ).unwrap();
	static ref STRIKE: Regex = Regex::new(r#"~~(.*?)~~"#         ).unwrap();
}

pub fn convert(s: &str) -> String {
	let s = LINK   .replace_all(&s, "[url=$2]$1[/url]"   );
	let s = CODE3  .replace_all(&s, "[code]$1[/code]"    );
	let s = CODE2  .replace_all(&s, "[code]$1[/code]"    );
	let s = CODE1  .replace_all(&s, "[code]$1[/code]"    );
	let s = H2     .replace_all(&s, "$1[b]$2[/b]"          );
	let s = H1     .replace_all(&s, "$1[h1]$2[/h1]"        );
	let s = BOLD   .replace_all(&s, "[b]$1[/b]"          );
	let s = ITALIC .replace_all(&s, "[i]$1[/i]"          );
	let s = STRIKE .replace_all(&s, "[strike]$1[/strike]");
	String::from(s)
}

#[test]
fn test() {
	assert_eq!(convert("\
[my url](google.com)
```
a
```
``a``
`a`
###test
##hello
#goodbye
**test**
*hi*
~~strike~~
"), "\
[url=google.com]my url[/url]
[code]
a
[/code]
[code]a[/code]
[code]a[/code]
[b]#test[/b]
[b]hello[/b]
[h1]goodbye[/h1]
[b]test[/b]
[i]hi[/i]
[strike]strike[/strike]
");
}
