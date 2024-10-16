pub struct TagNames<'a> {
    strong: &'a str,
    emphasis: &'a str,
    strike_through: &'a str,
    paragraph: &'a str,
    link: &'a str,
    code: &'a str,
    code_block: &'a str,
    br: &'a str,
    hr: &'a str,
    h1: &'a str,
    h2: &'a str,
    h3: &'a str,
    h4: &'a str,
    h5: &'a str,
    h6: &'a str,
}

impl<'a> TagNames<'a> {
    pub const fn strong(&self) -> &'a str {
        &self.strong
    }
    pub const fn emphasis(&self) -> &'a str {
        &self.emphasis
    }
    pub const fn strike_through(&self) -> &'a str {
        &self.strike_through
    }
    pub const fn paragraph(&self) -> &'a str {
        &self.paragraph
    }
    pub const fn link(&self) -> &'a str {
        &self.link
    }
    pub const fn code(&self) -> &'a str {
        &self.code
    }
    pub const fn code_block(&self) -> &'a str {
        &self.code_block
    }
    pub const fn br(&self) -> &'a str {
        &self.br
    }
    pub const fn hr(&self) -> &'a str {
        &self.hr
    }

    pub fn heading(&self, level: u8) -> &'a str {
        match level {
            1 => self.h1,
            2 => self.h2,
            3 => self.h3,
            4 => self.h4,
            5 => self.h5,
            6 => self.h6,
            _ => unreachable!(),
        }
    }
}

pub static DEFAULT_TAG_NAMES: TagNames<'static> = TagNames {
    strong: "$b",
    emphasis: "$i",
    strike_through: "$del",
    paragraph: "$p",
    link: "$link",
    code: "$code",
    code_block: "$codeBlock",
    br: "$br",
    hr: "$hr",
    h1: "$h1",
    h2: "$h2",
    h3: "$h3",
    h4: "$h4",
    h5: "$h5",
    h6: "$h6",
};
