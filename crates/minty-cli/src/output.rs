mod about;
mod bytes;
mod color;
mod comment;
mod entity;
mod icon;
mod list;
mod metadata;
mod num;
mod object;
mod post;
mod search_result;
mod tag;
mod text;
mod time;
mod user;
mod view;

pub use about::About;
pub use list::SliceExt;

use serde::Serialize;
use serde_json as json;
use std::io::{stderr, stdout, IsTerminal, Result, Write};

pub trait HumanReadable {
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()>;
}

impl<T> HumanReadable for Vec<T>
where
    T: HumanReadable,
{
    fn human_readable<W: Write>(&self, w: &mut W, indent: usize) -> Result<()> {
        self.iter()
            .try_for_each(|item| item.human_readable(w, indent))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Output {
    pub human_readable: bool,
    pub json: bool,
}

pub trait Print {
    fn print(&self, output: Output) -> Result<()>;
}

impl<T> Print for T
where
    T: Sized + Serialize + HumanReadable,
{
    fn print(&self, output: Output) -> Result<()> {
        let print_json = || println!("{}", json::to_string(self).unwrap());

        if output.json {
            print_json();

            if output.human_readable {
                self.human_readable(&mut stderr(), 0)?;
            }
        } else if output.human_readable || stdout().is_terminal() {
            self.human_readable(&mut stdout(), 0)?;
        } else {
            print_json();
        }

        Ok(())
    }
}
