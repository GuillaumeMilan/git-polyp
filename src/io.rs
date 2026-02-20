use colored::{ColoredString, Colorize};

pub struct YNQuestion {
    sentence: String,
}

impl YNQuestion {
    pub fn new(sentence: String) -> Self {
        YNQuestion { sentence }
    }

    pub fn ask(&self) -> Result<bool, std::io::Error> {
        println!("{} [y/n]", self.sentence);
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(matches!(input.trim().to_lowercase().as_str(), "y" | "yes"))
    }
}

pub trait Decorate: std::fmt::Display {
    fn deco_as_error(&self) -> String {
        let error_header = "[error]".bright_red().bold();
        format!("{} {}", error_header, self)
    }
    fn deco_as_command(&self) -> ColoredString {
        self.to_string().bright_blue()
    }
    fn deco_as_path(&self) -> ColoredString {
        self.to_string().deco_as_command().bright_green()
    }
}

impl Decorate for String {}
impl Decorate for &str {}
