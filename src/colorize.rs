#[cfg(target_os = "horizon")]
mod imp {
    pub trait Stylize {
        fn yellow(self) -> String;
        fn green(self) -> String;
        fn blue(self) -> String;
        fn red(self) -> String;
        fn grey(self) -> String;
    }

    impl Stylize for char {
        fn yellow(self) -> String {
            format!("\x1b[93m{}\x1b[0m", self)
        }
        fn green(self) -> String {
            format!("\x1b[92m{}\x1b[0m", self)
        }
        fn blue(self) -> String {
            format!("\x1b[36m{}\x1b[0m", self)
        }
        fn red(self) -> String {
            format!("\x1b[91m{}\x1b[0m", self)
        }
        fn grey(self) -> String {
            format!("\x1b[90m{}\x1b[0m", self)
        }
    }
}

#[cfg(not(target_os = "horizon"))]
mod imp {
    pub use crossterm::style::Stylize;
}

pub use imp::Stylize;