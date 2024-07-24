use crossterm::terminal;
use log::error;
use minty_core::Task;
use ratatui::{
    prelude::*,
    style::palette::tailwind,
    widgets::{block::Title, Block, Borders, Gauge},
    TerminalOptions, Viewport,
};
use std::io::{stdout, Stdout};
use tokio::{
    task::{self, JoinHandle},
    time::{sleep, Duration},
};

const VIEWPORT_HEIGHT: u16 = 2;

#[derive(Clone, Copy, Debug)]
struct Progress<'a> {
    title: &'a str,
    completed: usize,
    total: usize,
}

impl Widget for &Progress<'_> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        let completed = self.completed as f64;
        let total = self.total as f64;

        let ratio = completed / total;
        let percentage = (ratio * 100.0).round();

        let title = Block::default()
            .title(Title::from(self.title))
            .borders(Borders::NONE);

        Gauge::default()
            .gauge_style(tailwind::BLUE.c800)
            .ratio(ratio)
            .block(title)
            .label(format!("{completed}/{total} ({percentage}%)"))
            .render(area, buf);
    }
}

struct ProgressBar {
    title: String,
    task: Task,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    exited: bool,
}

impl ProgressBar {
    fn new(title: String, task: Task) -> Result<Self, String> {
        terminal::enable_raw_mode().map_err(|err| {
            format!("failed to enable terminal raw mode: {err}")
        })?;

        let terminal = Terminal::with_options(
            CrosstermBackend::new(stdout()),
            TerminalOptions {
                viewport: Viewport::Inline(VIEWPORT_HEIGHT),
            },
        )
        .map_err(|err| format!("failed to initialize terminal: {err}"))?;

        Ok(Self {
            title,
            task,
            terminal,
            exited: false,
        })
    }

    fn update(&mut self) -> Result<(), String> {
        let progress = Progress {
            title: self.title.as_str(),
            completed: self.task.completed(),
            total: self.task.total(),
        };

        self.terminal
            .draw(|frame| frame.render_widget(&progress, frame.size()))
            .map_err(|err| format!("failed to draw progress bar: {err}"))?;

        Ok(())
    }

    async fn run(mut self) -> Result<(), String> {
        loop {
            tokio::select! {
                biased;

                _ = self.task.cancelled() => {
                    break;
                }
                _ = sleep(Duration::from_millis(100)) => {
                    self.update()?;
                }
            }
        }

        // Briefly show the completed progress bar
        self.update()?;
        sleep(Duration::from_millis(500)).await;

        self.exit()
    }

    fn exit(&mut self) -> Result<(), String> {
        self.exited = true;

        terminal::disable_raw_mode().map_err(|err| {
            format!("failed to disable terminal raw mode: {err}")
        })?;

        self.terminal
            .clear()
            .map_err(|err| format!("failed to clear terminal: {err}"))
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if self.exited {
            return;
        }

        if let Err(err) = self.exit() {
            error!("Progress bar failed to exit properly: {err}");
        }
    }
}

pub struct ProgressBarTask {
    handle: JoinHandle<Result<(), String>>,
}

impl ProgressBarTask {
    pub fn new(title: String, task: Task) -> Result<Self, String> {
        let bar = ProgressBar::new(title, task)?;
        let handle = task::spawn(bar.run());

        Ok(Self { handle })
    }

    pub async fn join(self) -> Result<(), String> {
        self.handle
            .await
            .map_err(|err| format!("failed to join progress bar task: {err}"))?
    }
}
