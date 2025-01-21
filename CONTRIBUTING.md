### How python vs rust regexes

- [`re.match`](https://docs.python.org/3/library/re.html#re.match) searches at the beginning of a string
- [`re.search`](https://docs.python.org/3/library/re.html#re.search) searches anywhere at the string
- pip has [`regex`](https://github.com/mrabarnett/mrab-regex) package used here 
- [`regex::is_match`](https://docs.rs/regex/latest/regex/struct.Regex.html#method.is_match) has implicit `.*?` at the beginning and end of a pattern
- `fancy-regex` does not support branches during backtracking

When porting, be careful in tracking all regex usages. If `re.is_match` used, prepend the regular expression with `^`.
