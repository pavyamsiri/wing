use std::fmt;
use std::process::Command;
use std::time::Duration;

use jiff::tz::TimeZone;

use crate::duration::HumanDuration;

pub const TIMESTAMP_FORMAT: &'static str = "%Y-%m-%d: %I:%M:%S%P - %Z";

pub struct CommandDisplay<'a>(pub &'a Command);

impl<'a> fmt::Display for CommandDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let program = self.0.get_program().to_string_lossy();

        write!(f, "{}", program)?;

        for arg in self.0.get_args() {
            write!(f, " {}", arg.to_string_lossy())?;
        }

        Ok(())
    }
}

pub struct CommandReport {
    pub command: Command,
    pub elapsed: Duration,
    pub start: jiff::Timestamp,
    pub end: jiff::Timestamp,
}

impl fmt::Display for CommandReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Command: {}", CommandDisplay(&self.command))?;
        write!(f, "\n\tRan for:  {}.", HumanDuration(self.elapsed))?;
        write!(
            f,
            "\n\tStarted:  {}.",
            self.start
                .to_zoned(TimeZone::system())
                .strftime(TIMESTAMP_FORMAT)
        )?;
        write!(
            f,
            "\n\tFinished: {}.",
            self.end
                .to_zoned(TimeZone::system())
                .strftime(TIMESTAMP_FORMAT)
        )?;

        Ok(())
    }
}
