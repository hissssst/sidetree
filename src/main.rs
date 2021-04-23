#![feature(try_trait)]
#![feature(generators, generator_trait)]
mod commands;
mod file_tree;
mod prompt;
mod tree_entry;
mod util;

use crate::commands::App;
use crate::commands::Command;
use crate::commands::CommandQueue;
use crate::file_tree::{FileTree, FileTreeState};
use crate::prompt::Prompt;
use crate::prompt::StatusLine;
use crate::tree_entry::*;

use std::error::Error;
use std::io;
use std::path::PathBuf;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::Backend;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::Frame;
use tui::Terminal;

use crate::util::event::{Event, Events};

impl App {
  pub fn draw<B: Backend>(&mut self, f: &mut Frame<B>) {
    let chunks = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Min(0), Constraint::Length(1)].as_ref())
      .split(f.size());

    f.render_stateful_widget(FileTree::new(), chunks[0], &mut self.tree);
    self.statusline.draw(f, chunks[1]);
  }

  pub fn update(&mut self) {
    self.tree.update();
  }

  pub fn tick(&mut self) {
    self.run_queued_commands();
  }

  pub fn on_key(&mut self, k: Key) -> Option<()> {
    if self.statusline.has_focus() {
      self.statusline.on_key(&mut self.queued_commands, k);
      self.tree.update();
      return Some(());
    }
    match k {
      Key::Char('q') => {
        self.exit = true;
      }
      Key::Char('j') => {
        self.tree.select_next();
      }
      Key::Char('k') => {
        self.tree.select_prev();
      }
      Key::Char('\n') => {
        let entry = self.tree.entry_mut();
        if entry.is_dir {
          entry.toggle_expanded();
        } else {
          self.run_command(Command::Open(None))
        }
      }
      Key::Char('l') => {
        let entry = self.tree.entry_mut();
        if entry.is_dir {
          if !entry.expanded {
            entry.expand();
          } else {
            self.tree.select_next();
          }
        }
      }
      Key::Char('h') => {
        let entry = self.tree.entry_mut();
        if entry.expanded {
          entry.collapse();
        } else {
          self.tree.select_up();
        }
      }
      Key::Char('!') => {
        self.statusline.prompt(Box::new(ShellPrompt {}));
      }
      Key::Char(':') => {
        self.statusline.prompt(Box::new(CmdPrompt {}));
      }
      _ => {}
    }
    self.tree.update();
    Some(())
  }
}

fn main() -> Result<(), Box<dyn Error>> {
  // Terminal initialization
  let stdout = io::stdout().into_raw_mode()?;
  let stdout = MouseTerminal::from(stdout);
  let stdout = AlternateScreen::from(stdout);
  let backend = TermionBackend::new(stdout);
  let mut terminal = Terminal::new(backend)?;

  let mut events = Events::new();
  let mut app = App::new();
  loop {
    terminal.draw(|f| {
      app.draw(f);
    })?;

    if let Event::Input(input) = events.next()? {
      app.on_key(input);
      if app.statusline.has_focus() {
        events.disable_exit_key();
      } else {
        events.enable_exit_key();
      }
    }
    app.tick();
    if app.exit {
      break;
    }
  }

  Ok(())
}

pub struct ShellPrompt {}

impl Prompt for ShellPrompt {
  fn prompt_text(&self) -> &str {
    "!"
  }
  fn on_submit(&mut self, cmds: &mut CommandQueue, text: &str) {
    cmds.push(Command::Shell(text.to_string(), vec![]))
  }
  fn on_cancel(&mut self, _: &mut CommandQueue) {}
}

pub struct CmdPrompt {}

impl Prompt for CmdPrompt {
  fn prompt_text(&self) -> &str {
    ":"
  }
  fn on_submit(&mut self, cmds: &mut CommandQueue, text: &str) {
    cmds.push(Command::CmdStr(text.to_string()))
  }
  fn on_cancel(&mut self, _: &mut CommandQueue) {}
}
