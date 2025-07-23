use chrono::{Datelike, Local, NaiveDate, NaiveDateTime, Timelike, Weekday, TimeZone};
use chrono_tz::Tz;
use geocoding::{Openstreetmap, Point, Forward};
use tzf_rs::DefaultFinder;
use astro::{sun, lunar, time};
use std::f64::consts::PI;
use clap::Parser;

#[derive(Parser)]
#[command(name = "tdate")]
#[command(version = "0.1.1\n93 93/93\nDo what thou wilt shall be the whole of the Law.\nLove is the law, love under will.\n\nThanks to Lilith Vala Xara for the original implementation\nand JSKitty for the Rust port.")]
#[command(about = "Displays the current Thelemic date", long_about = None)]
struct Cli {
    /// Location for date calculation (e.g., "Las Vegas, NV")
    #[arg(short, long)]
    location: Option<String>,
    
    /// Date and time in format: year month day hour minute location
    #[arg(num_args = 6, value_names = &["YEAR", "MONTH", "DAY", "HOUR", "MINUTE", "LOCATION"])]
    datetime: Option<Vec<String>>,
    
    /// Hidden flag for Liber OZ
    #[arg(long = "oz", hide = true)]
    oz: bool,
}

struct ThelemicDate {
    finder: DefaultFinder,
}

impl ThelemicDate {
    fn new() -> Self {
        ThelemicDate {
            finder: DefaultFinder::new(),
        }
    }

    const NUMERALS: [&'static str; 23] = [
        "0", "i", "ii", "iii", "iv",
        "v", "vi", "vii", "viii", "ix",
        "x", "xi", "xii", "xiii", "xiv",
        "xv", "xvi", "xvii", "xviii", "xix",
        "xx", "xxi", "xxii"
    ];

    const SIGNS: [(&'static str, &'static str); 12] = [
        ("Aries", "♈"), ("Taurus", "♉"), ("Gemini", "♊"), ("Cancer", "♋"),
        ("Leo", "♌"), ("Virgo", "♍"), ("Libra", "♎"), ("Scorpio", "♏"),
        ("Sagittarius", "♐"), ("Capricorn", "♑"), ("Aquarius", "♒"), ("Pisces", "♓")
    ];

    const DAYS_OF_WEEK: [&'static str; 7] = [
        "Lunae", "Martis", "Mercurii", "Jovis",
        "Veneris", "Saturnii", "Solis"
    ];

    fn get_geopos(&self, location: &str) -> Result<(f64, f64, String), Box<dyn std::error::Error>> {
        let osm = Openstreetmap::new();
        let res: Vec<Point<f64>> = osm.forward(location)?;
        
        if let Some(point) = res.first() {
            let lat = point.y();
            let lon = point.x();
            let timezone_name = self.finder.get_tz_name(lon, lat);
            Ok((lat, lon, timezone_name.to_string()))
        } else {
            Err("Location not found".into())
        }
    }

    fn get_timezone(&self, location: &str) -> Result<Tz, Box<dyn std::error::Error>> {
        let (_, _, tz_name) = self.get_geopos(location)?;
        tz_name.parse::<Tz>()
            .map_err(|e| format!("Invalid timezone: {}", e).into())
    }

    fn get_sign_from_longitude(&self, longitude: f64) -> &'static str {
        let degree = longitude * 180.0 / PI;
        let normalized_degree = if degree < 0.0 { degree + 360.0 } else { degree };
        let sign_index = (normalized_degree / 30.0) as usize;
        Self::SIGNS[sign_index % 12].0
    }

    fn get_degree_in_sign(&self, longitude: f64) -> i32 {
        let degree = longitude * 180.0 / PI;
        let normalized_degree = if degree < 0.0 { degree + 360.0 } else { degree };
        (normalized_degree % 30.0) as i32
    }

    fn weekday_to_index(weekday: Weekday) -> usize {
        match weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        }
    }

    pub fn now(&self, location: &str) -> Result<String, Box<dyn std::error::Error>> {
        let (_lat, _lon, _tz_name) = self.get_geopos(location)?;
        let tz = self.get_timezone(location)?;
        let now = Local::now().with_timezone(&tz);
        
        let ve_year = now.year();
        let ve_years_total = ve_year - 1904;
        let cycle_i = ve_years_total / 22;
        let cycle_ii = ve_year - 1904 - (cycle_i * 22);
        let na_year = format!("{}{}", 
            Self::NUMERALS[cycle_i as usize].to_uppercase(), 
            Self::NUMERALS[cycle_ii as usize]
        );
        
        let ve_weekday = Self::weekday_to_index(now.weekday());
        
        // Calculate Julian Day
        let naive_utc = now.naive_utc();
        let jd = time::julian_day(
            &time::Date {
                year: naive_utc.year() as i16,
                month: naive_utc.month() as u8,
                decimal_day: naive_utc.day() as f64 
                    + naive_utc.hour() as f64 / 24.0 
                    + naive_utc.minute() as f64 / 1440.0 
                    + naive_utc.second() as f64 / 86400.0,
                cal_type: time::CalType::Gregorian,
            }
        );
        
        // Get sun position
        let (sun_pos, _sun_dist) = sun::geocent_ecl_pos(jd);
        let sun_sign = self.get_sign_from_longitude(sun_pos.long);
        let sun_degree = self.get_degree_in_sign(sun_pos.long);
        
        // Get moon position
        let (moon_pos, _moon_dist) = lunar::geocent_ecl_pos(jd);
        let moon_sign = self.get_sign_from_longitude(moon_pos.long);
        let moon_degree = self.get_degree_in_sign(moon_pos.long);
        
        Ok(format!(
            "☉ in {}º {} : ☽ in {}º {} : dies {} : Anno {} æræ legis",
            sun_degree, sun_sign,
            moon_degree, moon_sign,
            Self::DAYS_OF_WEEK[ve_weekday],
            na_year
        ))
    }

    pub fn in_day(&self, year: i32, month: u32, day: u32, hour: u32, minute: u32, location: &str) 
        -> Result<String, Box<dyn std::error::Error>> {
        let (_lat, _lon, _tz_name) = self.get_geopos(location)?;
        let tz = self.get_timezone(location)?;
        
        let date = NaiveDate::from_ymd_opt(year, month, day)
            .ok_or("Invalid date")?;
        let ve_weekday = Self::weekday_to_index(date.weekday());
        
        // Adjust year for Thelemic calendar (New Year starts at spring equinox ~March 20)
        let ve_in_day_na_year = if month < 3 || (month == 3 && day < 20) {
            year - 1905
        } else {
            year - 1904
        };
        
        let cycle_i = ve_in_day_na_year / 22;
        let cycle_ii = ve_in_day_na_year - (cycle_i * 22);
        let na_year = format!("{}{}", 
            Self::NUMERALS[cycle_i as usize].to_uppercase(), 
            Self::NUMERALS[cycle_ii as usize]
        );
        
        // Create datetime
        let naive_dt = NaiveDateTime::new(
            date,
            chrono::NaiveTime::from_hms_opt(hour, minute, 0).ok_or("Invalid time")?
        );
        let dt = tz.from_local_datetime(&naive_dt)
            .single()
            .ok_or("Ambiguous local time")?;
        
        // Calculate Julian Day
        let naive_utc = dt.naive_utc();
        let jd = time::julian_day(
            &time::Date {
                year: naive_utc.year() as i16,
                month: naive_utc.month() as u8,
                decimal_day: naive_utc.day() as f64 
                    + naive_utc.hour() as f64 / 24.0 
                    + naive_utc.minute() as f64 / 1440.0 
                    + naive_utc.second() as f64 / 86400.0,
                cal_type: time::CalType::Gregorian,
            }
        );
        
        // Get sun position
        let (sun_pos, _sun_dist) = sun::geocent_ecl_pos(jd);
        let sun_sign = self.get_sign_from_longitude(sun_pos.long);
        let sun_degree = self.get_degree_in_sign(sun_pos.long);
        
        // Get moon position
        let (moon_pos, _moon_dist) = lunar::geocent_ecl_pos(jd);
        let moon_sign = self.get_sign_from_longitude(moon_pos.long);
        let moon_degree = self.get_degree_in_sign(moon_pos.long);
        
        Ok(format!(
            "☉ in {}º {} : ☽ in {}º {} : dies {} : Anno {} æræ legis",
            sun_degree, sun_sign,
            moon_degree, moon_sign,
            Self::DAYS_OF_WEEK[ve_weekday],
            na_year
        ))
    }
}

fn print_liber_oz() {
    println!(r#"
LIBER LXXVII
OZ

"the law of
the strong:
this is our law
and the joy
of the world." AL. II. 21

"Do what thou wilt shall be the whole of the Law." --AL. I. 40

"thou hast no right but to do thy will. Do that, and no other shall say nay." --AL. I. 42-3

"Every man and every woman is a star." --AL. I. 3

There is no god but man.

1. Man has the right to live by his own law--
        to live in the way that he wills to do:
        to work as he will:
        to play as he will:
        to rest as he will:
        to die when and how he will.

2. Man has the right to eat what he will:
        to drink what he will:
        to dwell where he will:
        to move as he will on the face of the earth.

3. Man has the right to think what he will:
        to speak what he will:
        to write what he will:
        to draw, paint, carve, etch, mould, build as he will:
        to dress as he will.

4. Man has the right to love as he will:--
        "take your fill and will of love as ye will,
        when, where, and with whom ye will." --AL. I. 51

5. Man has the right to kill those who would thwart these rights.

"the slaves shall serve." --AL. II. 58

"Love is the law, love under will." --AL. I. 57
"#);
}

fn main() {
    let cli = Cli::parse();
    
    // Check for Liber OZ flag first
    if cli.oz {
        print_liber_oz();
        return;
    }
    
    let date_data = ThelemicDate::new();
    
    if let Some(datetime_args) = cli.datetime {
        // Handle specific date/time
        if datetime_args.len() != 6 {
            eprintln!("Error: Expected 6 arguments for datetime (year month day hour minute location)");
            std::process::exit(1);
        }
        
        let year: i32 = datetime_args[0].parse().expect("Invalid year");
        let month: u32 = datetime_args[1].parse().expect("Invalid month");
        let day: u32 = datetime_args[2].parse().expect("Invalid day");
        let hour: u32 = datetime_args[3].parse().expect("Invalid hour");
        let minute: u32 = datetime_args[4].parse().expect("Invalid minute");
        let location = &datetime_args[5];
        
        match date_data.in_day(year, month, day, hour, minute, location) {
            Ok(result) => println!("{}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    } else {
        // Handle current date/time
        let location = cli.location.as_deref().unwrap_or("Las Vegas, NV");
        match date_data.now(location) {
            Ok(current_date) => println!("{}", current_date),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}
