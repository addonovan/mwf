use view::View;

pub trait Decorator
{
    fn decorate(&self, view: View) -> View;
}

pub struct Markdown;
impl Decorator for Markdown
{
    fn decorate(&self, view: View) -> View
    {
        use pulldown_cmark::{Parser, html};

        let mut output = String::new();
        let p = Parser::new(&view.content);
        html::push_html(&mut output, p);

        // create a new view with the html output and the correct
        // mime type
        View {
            content: output,
            mime: "text/html".parse().unwrap(),
        }
    }
}

pub struct Surround
{
    pre: String,
    post: String,
}

impl Surround
{
    pub fn new<T: Into<String>, U: Into<String>>(pre: T, post: U) -> Self
    {
        Surround {
            pre: pre.into(),
            post: post.into(),
        }
    }

    pub fn from<T: Into<String>>(input: T) -> Self
    {
        let input = input.into();

        let mut vec: Vec<String> = input.split("{{middle}}")
            .take(2)
            .map(String::from)
            .collect();

        let post = vec.pop().unwrap_or("".into());
        let pre = vec.pop().unwrap_or("".into());

        Surround {
            pre,
            post
        }
    }
}

impl Decorator for Surround
{
    fn decorate(&self, view: View) -> View
    {
        View {
            content: format!("{}{}{}", self.pre, view.content, self.post),
            mime: view.mime,
        }
    }
}
