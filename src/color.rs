// rasmx86_64 - color.rs
// ---------------------
// made by matissoss
// licensed under MPL 2.0
pub enum BaseColor{
    BLACK   ,
    RED     ,
    GREEN   ,
    BLUE    ,
    YELLOW  ,
    PURPLE  ,
    CYAN    ,
    WHITE
}

pub enum Color {
    BOLD(BaseColor),
    REGULAR(BaseColor),
    UNDERLINE(BaseColor),
}

pub struct ColorString {
    vl: String,
    cl: Color,
}

pub trait ColorText{
    fn black(&self)         -> ColorString;
    fn bold_black(&self)    -> ColorString;
    fn red(&self)           -> ColorString;
    fn bold_red(&self)      -> ColorString;
    fn green(&self)         -> ColorString;
    fn bold_green(&self)    -> ColorString;
    fn yellow(&self)        -> ColorString;
    fn bold_yellow(&self)   -> ColorString;
    fn blue(&self)          -> ColorString;
    fn bold_blue(&self)     -> ColorString;
    fn purple(&self)        -> ColorString;
    fn bold_purple(&self)   -> ColorString;
    fn cyan(&self)          -> ColorString;
    fn bold_cyan(&self)     -> ColorString;
}

impl ColorText for &str{
    fn black(&self)         -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::REGULAR(BaseColor::BLACK)
        }
    }
    fn bold_black(&self)    -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::BOLD(BaseColor::BLACK)
        }
    }
    fn red(&self)           -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::REGULAR(BaseColor::RED)
        }
    }
    fn bold_red(&self)      -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::BOLD(BaseColor::RED)
        }
    }
    fn green(&self)         -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::REGULAR(BaseColor::GREEN)
        }
    }
    fn bold_green(&self)    -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::BOLD(BaseColor::GREEN)
        }
    }
    fn yellow(&self)        -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::REGULAR(BaseColor::YELLOW)
        }
    }
    fn bold_yellow(&self)   -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::BOLD(BaseColor::YELLOW)
        }
    }
    fn blue(&self)          -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::REGULAR(BaseColor::BLUE)
        }
    }
    fn bold_blue(&self)     -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::BOLD(BaseColor::BLUE)
        }
    }
    fn purple(&self)        -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::REGULAR(BaseColor::PURPLE)
        }
    }
    fn bold_purple(&self)   -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::BOLD(BaseColor::PURPLE)
        }
    }
    fn cyan(&self)          -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::REGULAR(BaseColor::CYAN)
        }
    }
    fn bold_cyan(&self)     -> ColorString{
        ColorString{
            vl: self.to_string(),
            cl: Color::BOLD(BaseColor::CYAN)
        }
    }
}

impl std::fmt::Display for ColorString{
    fn fmt(&self, formt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error>{
        match &self.cl {
            Color::REGULAR(cl) => {
                match cl {
                    BaseColor::BLACK  => write!(formt, "\x1b[0;30m{}\x1b[0m", self.vl),
                    BaseColor::RED    => write!(formt, "\x1b[0;31m{}\x1b[0m", self.vl),
                    BaseColor::GREEN  => write!(formt, "\x1b[0;32m{}\x1b[0m", self.vl),
                    BaseColor::YELLOW => write!(formt, "\x1b[0;33m{}\x1b[0m", self.vl),
                    BaseColor::BLUE   => write!(formt, "\x1b[0;34m{}\x1b[0m", self.vl),
                    BaseColor::PURPLE => write!(formt, "\x1b[0;35m{}\x1b[0m", self.vl),
                    BaseColor::CYAN   => write!(formt, "\x1b[0;36m{}\x1b[0m", self.vl),
                    BaseColor::WHITE  => write!(formt, "\x1b[0;37m{}\x1b[0m", self.vl),
                }
            },
            Color::BOLD(cl) => {
                match cl {
                    BaseColor::BLACK  => write!(formt, "\x1b[1;30m{}\x1b[0m", self.vl),
                    BaseColor::RED    => write!(formt, "\x1b[1;31m{}\x1b[0m", self.vl),
                    BaseColor::GREEN  => write!(formt, "\x1b[1;32m{}\x1b[0m", self.vl),
                    BaseColor::YELLOW => write!(formt, "\x1b[1;33m{}\x1b[0m", self.vl),
                    BaseColor::BLUE   => write!(formt, "\x1b[1;34m{}\x1b[0m", self.vl),
                    BaseColor::PURPLE => write!(formt, "\x1b[1;35m{}\x1b[0m", self.vl),
                    BaseColor::CYAN   => write!(formt, "\x1b[1;36m{}\x1b[0m", self.vl),
                    BaseColor::WHITE  => write!(formt, "\x1b[1;37m{}\x1b[0m", self.vl),
                }
            }
            Color::UNDERLINE(cl) => {
                match cl {
                    BaseColor::BLACK  => write!(formt, "\x1b[4;30m{}\x1b[0m", self.vl),
                    BaseColor::RED    => write!(formt, "\x1b[4;31m{}\x1b[0m", self.vl),
                    BaseColor::GREEN  => write!(formt, "\x1b[4;32m{}\x1b[0m", self.vl),
                    BaseColor::YELLOW => write!(formt, "\x1b[4;33m{}\x1b[0m", self.vl),
                    BaseColor::BLUE   => write!(formt, "\x1b[4;34m{}\x1b[0m", self.vl),
                    BaseColor::PURPLE => write!(formt, "\x1b[4;35m{}\x1b[0m", self.vl),
                    BaseColor::CYAN   => write!(formt, "\x1b[4;36m{}\x1b[0m", self.vl),
                    BaseColor::WHITE  => write!(formt, "\x1b[4;37m{}\x1b[0m", self.vl),
                }
            }
        }
    }
}
