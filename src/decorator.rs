use view::View;

/// A generic trait for anything which can decorate a [view](View) in some way.
///
/// A decorator will take in a pre-existing view and generate a new one
/// (or possibly modify the existing one in-place). This can be used for
/// a wide variety of things, such as converting markdown to html or
/// applying a certain style to the content of a page, or both!
///
/// Because a decorator is required to return a view, these calls may be
/// chained by calling view's `apply` method.
pub trait Decorator
{
    /// Applies this decorator to the given `view`, altering it or creating
    /// a fresh one.
    fn decorate(&self, view: View) -> View;
}

/// A decorator which treats the text in the view as markdown and generates
/// HTML from it. This will also alter the mime type of the view, changing it
/// to `text/html`.
///
/// ```rust
/// use mwf::{View, decorator};
///
/// let dec = decorator::Markdown;
/// let view = View::raw("# Hello world!").apply(&dec);
///
/// assert_eq!("<h1>Hello world!</h1>\n", view.content);
/// assert_eq!("text", view.mime.type_());
/// assert_eq!("html", view.mime.subtype());
/// ```
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

/// A decorator whose goal is to surround the text of the view with
/// preset leading and trailing strings.
pub struct Surround
{
    pre: String,
    post: String,
}

impl Surround
{
    /// Creates a new decorator which will prefix the view's content with
    /// `pre` and then append `post` onto the end of the content.
    ///
    /// ```rust
    /// use mwf::{View, decorator};
    ///
    /// let dec = decorator::Surround::new("foo", "baz");
    /// let view = View::raw("bar").apply(&dec);
    ///
    /// assert_eq!("foobarbaz", view.content);
    /// ```
    pub fn new<T: Into<String>, U: Into<String>>(pre: T, post: U) -> Self
    {
        Surround {
            pre: pre.into(),
            post: post.into(),
        }
    }

    /// Creates a new decorator which will surround the view's content with the
    /// given `input`, after it has been split on the delimiter `{{middle}}`.
    /// If the delimiter is not found, then the text will all be *added to the
    /// end* of the view's content.
    ///
    /// ```rust
    /// use mwf::{View, decorator};
    ///
    /// // with a {{middle}} tag
    /// let dec = decorator::Surround::from("foo{{middle}}baz");
    /// let view = View::raw("bar").apply(&dec);
    ///
    /// assert_eq!("foobarbaz", view.content);
    ///
    /// // with no {{middle}} tag
    /// let dec = decorator::Surround::from("barbaz");
    /// let view = View::raw("foo").apply(&dec);
    ///
    /// assert_eq!("foobarbaz", view.content);
    /// ```
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

#[cfg(test)]
mod test
{
    use super::*;

    /// Test if the [Surround::new] API accepts two `&'static str`s as
    /// parameters and makes a view correctly from them.
    #[test]
    fn surround_new_strs()
    {
        let dec = Surround::new("foo", "baz");
        let view = View::raw("bar").apply(&dec);
        assert_eq!("foobarbaz", view.content);
    }

    /// Test if the [Surround::new] API accepts two `String`s as parameters.
    #[test]
    fn surround_new_strings()
    {
        let dec = Surround::new("foo".to_string(), "baz".to_string());
        let view = View::raw("bar").apply(&dec);
        assert_eq!("foobarbaz", view.content);
    }

    /// Test if the [Surround::new] API can mix and match between
    /// `&'static str`s and `String`s
    #[test]
    fn surround_new_mixed()
    {
        let dec = Surround::new("foo".to_string(), "baz");
        let view = View::raw("bar").apply(&dec);
        assert_eq!("foobarbaz", view.content);

        let dec = Surround::new("foo", "baz".to_string());
        let view = View::raw("bar").apply(&dec);
        assert_eq!("foobarbaz", view.content);
    }

    /// Tests the [Surround::from] API's ability to split on the `{{middle}}`
    /// delimiter and still applies to a view correctly.
    #[test]
    fn surround_from_with_delimiter()
    {
        let dec = Surround::from("foo{{middle}}baz");
        let view = View::raw("bar").apply(&dec);
        assert_eq!("foobarbaz", view.content);
    }

    /// Tests the [Surround::from] API's ability to handle the case when there
    /// is no delimiter in the given string.
    #[test]
    fn surround_from_without_delimiter()
    {
        let dec = Surround::from("barbaz");
        let view = View::raw("foo").apply(&dec);
        assert_eq!("foobarbaz", view.content);
    }

    /// Tests the [Surround::from] API's ability to handle the case when there
    /// are multiple delimiters in the given string.
    #[test]
    fn surround_from_with_multiple_delimiters()
    {
        let dec = Surround::from("foo{{middle}}baz{{middle}}quz");
        let view = View::raw("bar").apply(&dec);
        assert_eq!("foobarbaz", view.content);
    }

    /// Tests the [Markdown] decorator's ability to correctly convert markdown
    /// source into html, and also change the mime type to `text/html`.
    #[test]
    fn markdown()
    {
        let dec = Markdown;
        let view = View::raw("# Hello world!").apply(&dec);
        assert_eq!("<h1>Hello world!</h1>\n", view.content);
        assert_eq!("text", view.mime.type_());
        assert_eq!("html", view.mime.subtype());
    }

    /// Tests the ability to chain [View::apply] functions together and have
    /// each [Decorator] work together.
    #[test]
    fn apply_chaining()
    {
        let surround = Surround::new("*foo", "baz*");
        let md = Markdown;
        let view = View::raw("bar").apply(&surround).apply(&md);
        assert_eq!("<p><em>foobarbaz</em></p>\n", view.content);
        assert_eq!("text", view.mime.type_());
        assert_eq!("html", view.mime.subtype());
    }
}
