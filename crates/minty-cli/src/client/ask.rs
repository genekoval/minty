use crate::{Error, Result};

use minty::{EntityProfile, Source};
use std::{
    fmt::{self, Display},
    io::{stdin, stdout, Write},
    mem,
};

#[derive(Default)]
struct SourceChoice {
    id: i64,
    url: String,
}

impl From<Source> for SourceChoice {
    fn from(value: Source) -> Self {
        Self {
            id: value.id,
            url: value.url.to_string(),
        }
    }
}

impl Display for SourceChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.url)
    }
}

pub fn delete_alias(profile: EntityProfile) -> Result<Option<String>> {
    println!("{}", profile.name);
    multiple_choice(profile.aliases, "Which alias would you like to remove?")
}

pub fn delete_source(
    entity: &'static str,
    profile: EntityProfile,
) -> Result<Option<i64>> {
    if profile.sources.is_empty() {
        println!("{entity} '{}' has no links", profile.name);
        return Ok(None);
    }

    let sources: Vec<SourceChoice> =
        profile.sources.into_iter().map(Into::into).collect();

    let source =
        multiple_choice(sources, "Which source would you like to remove?")?
            .map(|source| source.id);

    Ok(source)
}

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
