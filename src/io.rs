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
