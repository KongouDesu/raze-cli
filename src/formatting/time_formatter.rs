use std::fmt;
use time;

#[derive(Debug)]
pub struct Time {
    days: i64,
    hours: i32,
    minutes: i32,
    seconds: i32,
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} days, {} hours, {} minutes, {} seconds", self.days, self.hours, self.minutes, self.seconds)
    }
}

const SECONDS_PER_MINUTE: f64 = 60.;
const SECONDS_PER_HOUR: f64 = SECONDS_PER_MINUTE*60.;
const SECONDS_PER_DAY: f64 = SECONDS_PER_HOUR*24.;


// Compares a given unix timestamp to the current time
// Returns a Time struct, containing the delta time
pub fn time_since_timestamp(secs: i64) -> Time {
    let diff = (time::get_time().sec - secs) as f64;
    Time {
        days: (diff/SECONDS_PER_DAY) as i64,
        hours: ((diff % SECONDS_PER_DAY)/SECONDS_PER_HOUR) as i32,
        minutes: ((diff % SECONDS_PER_HOUR)/SECONDS_PER_MINUTE) as i32,
        seconds: (diff % 60.) as i32,
    }
}