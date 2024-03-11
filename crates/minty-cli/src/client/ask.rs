use crate::{Error, Result};

use std::{
    fmt::Display,
    io::{stdin, stdout, Write},
    mem,
};

pub fn multiple_choice<T>(
    mut choices: Vec<T>,
    prompt: &str,
) -> Result<Option<T>>
where
    T: Default + Display,
{
    if choices.is_empty() {
        return Ok(None);
    }

    choices.iter().enumerate().for_each(|(i, choice)| {
        let i = i + 1;
        println!("{i}. {choice}");
    });

    let line = readline(prompt)?;
    let line = line.trim();
    if line.is_empty() {
        return Ok(None);
    }

    let choice = line
        .parse::<usize>()
        .ok()
        .and_then(|i| {
            let i = i.checked_sub(1)?;
            let choice = choices.get_mut(i)?;
            Some(mem::take(choice))
        })
        .ok_or_else(|| format!("invalid choice: {line}"))?;

    Ok(Some(choice))
}

fn readline(prompt: &str) -> Result<String> {
    write!(stdout(), "{prompt} ")?;
    stdout().flush()?;

    let mut buffer = String::new();
    stdin().read_line(&mut buffer)?;

    Ok(buffer)
}

macro_rules! confirm {
    ($e:expr) => {
        match $crate::client::ask::inner::confirm($e) {
            Ok(confirmed) => match confirmed {
                true => Ok(()),
                false => return Ok(()),
            },
            Err(err) => Err(err),
        }
    };
}

pub(crate) use confirm;

pub mod inner {
    use super::*;

    pub fn confirm(prompt: &str) -> Result<bool> {
        let line = readline(prompt)?.trim().to_lowercase();
        if line.is_empty() {
            return Ok(false);
        }

        if line == "yes" || line == "y" {
            Ok(true)
        } else if line == "no" || line == "n" {
            Ok(false)
        } else {
            Err(Error::Other(format!("unrecognized option: {line}")))
        }
    }
}
