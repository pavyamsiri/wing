use colored::Colorize;
use std::fmt;
use std::time::Duration;

pub struct HumanDuration(pub Duration);
pub struct ColoredDuration(pub Duration);

impl fmt::Display for HumanDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut written = false;
        macro_rules! write_unit {
            ($value:expr, $suffix:expr) => {
                if $value > 0 {
                    if written {
                        write!(f, " ")?;
                    }
                    write!(f, "{}{}", $value.to_string(), $suffix)?;
                    written = true;
                }
            };
        }

        let duration = self.0;
        let total_nanos = duration.as_nanos();
        let total_micros = duration.as_micros();
        let total_millis = duration.as_millis();
        let total_secs = duration.as_secs();

        let days = total_secs / 86_400;
        let hours = (total_secs % 86_400) / 3_600;
        let minutes = (total_secs % 3_600) / 60;
        let seconds = total_secs % 60;
        let millis = (total_millis % 1_000) as u64;
        let micros = (total_micros % 1_000) as u64;
        let nanos = (total_nanos % 1_000) as u64;

        write_unit!(days, "d");
        write_unit!(hours, "h");
        write_unit!(minutes, "m");
        write_unit!(seconds, "s");
        write_unit!(millis, "ms");
        write_unit!(micros, "µs");
        write_unit!(nanos, "ns");

        if !written {
            write!(f, "{}", "0s".green())?;
        }

        Ok(())
    }
}

impl fmt::Display for ColoredDuration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut written = false;
        macro_rules! write_unit {
            ($value:expr, $suffix:expr, $color:ident) => {
                if $value > 0 {
                    if written {
                        write!(f, " ")?;
                    }
                    write!(
                        f,
                        "{}{}",
                        $value.to_string().bold().$color(),
                        $suffix.$color()
                    )?;
                    written = true;
                }
            };
        }

        let duration = self.0;
        let total_nanos = duration.as_nanos();
        let total_micros = duration.as_micros();
        let total_millis = duration.as_millis();
        let total_secs = duration.as_secs();

        let days = total_secs / 86_400;
        let hours = (total_secs % 86_400) / 3_600;
        let minutes = (total_secs % 3_600) / 60;
        let seconds = total_secs % 60;
        let millis = (total_millis % 1_000) as u64;
        let micros = (total_micros % 1_000) as u64;
        let nanos = (total_nanos % 1_000) as u64;

        write_unit!(days, "d", red);
        write_unit!(hours, "h", magenta);
        write_unit!(minutes, "m", yellow);
        write_unit!(seconds, "s", green);
        write_unit!(millis, "ms", cyan);
        write_unit!(micros, "µs", blue);
        write_unit!(nanos, "ns", purple);

        if !written {
            write!(f, "{}", "0s".green())?;
        }

        Ok(())
    }
}
