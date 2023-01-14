use std::cmp::Ordering;

fn match_metric(pattern: &str, text: &str) -> f32 {
  let matches = pattern.chars().zip(text.chars()).take_while(|(x, y)| x == y).count();
  matches as f32 / pattern.len() as f32
}

#[derive(Debug, Clone)]
pub enum FtpCmd {
  USER(String),
  PASS(String),
  PWD,
  CWD(String),
  PASV,
  LIST(String),
  RETR(String),
  QUIT,
}

#[derive(Debug, Clone)]
pub struct FtpCmdFuzz {
  matched_cmd: FtpCmd,
  match_rate: f32,
}

impl FtpCmdFuzz {
  fn new(matched_cmd: FtpCmd, match_rate: f32) -> Self {
    Self {
      matched_cmd,
      match_rate,
    }
  }
  fn some_new(matched_cmd: FtpCmd, match_rate: f32) -> Option<Self> {
    Some(Self::new(matched_cmd, match_rate))
  }

  fn cmp(a: &Self, b: &Self) -> Ordering {
    PartialOrd::partial_cmp(&a.match_rate, &b.match_rate).unwrap_or(Ordering::Equal)
  }

  pub fn is_exact(&self) -> bool {
    self.match_rate == 1.0
  }

  pub fn cmd(&self) -> FtpCmd {
    self.matched_cmd.clone()
  }
}

impl FtpCmd {
  pub fn is_pasv(&self) -> bool {
    matches!(self, FtpCmd::LIST(_) | FtpCmd::RETR(_))
  }

  pub fn as_ftp_request(&self) -> String {
    match self {
      FtpCmd::USER(name) => format!("USER {name}"),
      FtpCmd::PASS(password) => format!("PASS {password}"),
      FtpCmd::PWD => "PWD".into(),
      FtpCmd::CWD(dir) => format!("CWD {dir}"),
      FtpCmd::PASV => "PASV".into(),
      FtpCmd::LIST(dir) => {
        let sep = if dir.is_empty() { "" } else { " " };
        format!("LIST{sep}{dir}")
      }
      FtpCmd::RETR(file) => format!("RETR {file}"),
      FtpCmd::QUIT => "QUIT".into(),
    }
  }

  fn parse_user(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    let (cmd, uname) = raw_cmd.split_once(" ")?;
    FtpCmdFuzz::some_new(FtpCmd::USER(uname.into()), match_metric("USER", cmd))
  }
  fn parse_pass(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    if let Some((cmd, pwd)) = raw_cmd.split_once(" ") {
      FtpCmdFuzz::some_new(FtpCmd::PASS(pwd.into()), match_metric("PASS", cmd))
    } else {
      FtpCmdFuzz::some_new(FtpCmd::PASS("".into()), match_metric("PASS", raw_cmd))
    }
  }
  fn parse_cwd(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    let (cmd, dir) = raw_cmd.split_once(" ")?;
    FtpCmdFuzz::some_new(FtpCmd::CWD(dir.into()), match_metric("CWD", cmd))
  }
  fn parse_list(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    if let Some((cmd, dir)) = raw_cmd.split_once(" ") {
      FtpCmdFuzz::some_new(FtpCmd::LIST(dir.into()), match_metric("LIST", cmd))
    } else {
      FtpCmdFuzz::some_new(FtpCmd::LIST("".into()), match_metric("LIST", raw_cmd))
    }
  }
  fn parse_retr(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    let (cmd, file) = raw_cmd.split_once(" ")?;
    FtpCmdFuzz::some_new(FtpCmd::RETR(file.to_string()), match_metric("RETR", cmd))
  }
  fn parse_pwd(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    FtpCmdFuzz::some_new(FtpCmd::PWD, match_metric("PWD", raw_cmd))
  }
  fn parse_pasv(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    FtpCmdFuzz::some_new(FtpCmd::PASV, match_metric("PASV", raw_cmd))
  }
  fn parse_quit(raw_cmd: &str) -> Option<FtpCmdFuzz> {
    FtpCmdFuzz::some_new(FtpCmd::QUIT, match_metric("QUIT", raw_cmd))
  }

  pub fn parse(raw_cmd: &str) -> Vec<FtpCmdFuzz> {
    let mut ret = vec![];
    fn add_one<F: Fn(&str) -> Option<FtpCmdFuzz>>(parse_func: F, cmd: &str, ret: &mut Vec<FtpCmdFuzz>) {
      if let Some(parse_ret) = parse_func(cmd).filter(|x| x.match_rate != 0.0) {
        ret.push(parse_ret);
      }
    }
    let raw_cmd = raw_cmd.trim();
    add_one(Self::parse_user, raw_cmd, &mut ret);
    add_one(Self::parse_pass, raw_cmd, &mut ret);
    add_one(Self::parse_pwd, raw_cmd, &mut ret);
    add_one(Self::parse_cwd, raw_cmd, &mut ret);
    add_one(Self::parse_pasv, raw_cmd, &mut ret);
    add_one(Self::parse_list, raw_cmd, &mut ret);
    add_one(Self::parse_retr, raw_cmd, &mut ret);
    add_one(Self::parse_quit, raw_cmd, &mut ret);

    ret.sort_by(FtpCmdFuzz::cmp);
    ret.reverse();
    ret
  }
}
