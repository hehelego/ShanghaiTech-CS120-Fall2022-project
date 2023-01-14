use console::{style, StyledObject};
use std::fmt::Debug;
use std::io::{stdin, stdout, Result, Write};

pub fn getline() -> Result<String> {
  let mut line = String::new();
  stdin().read_line(&mut line)?;
  Ok(line)
}

pub fn flush_stdout() -> Result<()> {
  stdout().flush()
}

pub fn pick<T: Debug + Clone>(options: &[T]) -> Option<T> {
  println!("{}", note_prompt("no exact match, pick fuzzy match results"));
  if options.is_empty() {
    println!("{}", note_prompt("no fuzzy match candidate"));
    return None;
  }

  for (idx, opt) in options.iter().enumerate() {
    println!("{} {:?}", note_prompt(format!("---choice[{idx}]")), opt)
  }
  print!("{}", note_prompt("pick> "));
  stdout().flush().unwrap();

  let line = getline().ok()?;
  let idx = line.trim().parse::<usize>().ok()?;

  options.get(idx).cloned()
}

pub fn cmd_prompt<D>(prompt: D) -> StyledObject<D> {
  style(prompt).green().bold()
}
pub fn resp_prompt<D>(prompt: D) -> StyledObject<D> {
  style(prompt).red().bold()
}
pub fn note_prompt<D>(prompt: D) -> StyledObject<D> {
  style(prompt).yellow().dim()
}
