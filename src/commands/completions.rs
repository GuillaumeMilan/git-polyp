use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

use crate::GitPolyp;

pub fn run(shell: Shell) {
    let mut cmd = GitPolyp::command();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut io::stdout());
}
