use std::{fs, io::Read};
use flate2::read::GzDecoder;
use plotters::prelude::*;

/*  
SWEAT - Strange WEather in AusTin
This program will statistically evaluate whether thweather in Austin is
strange or not (compared to other locations).
*/

fn main() {
    /*
    Required steps:
        0. Acquire data from the website
        1. Parse the info and extract all data for every line
        2. Validate all info
        3. Process data:
            a. Mean + standard deviation for each day, week, month
            b. Graph data
    * This should be done with a web deployment of the service in mind
    */
    // let token = fs::read_to_string("./token")
    //     .expect("Could not read token form file");
    let year = "2022";
    // camp mabry : 13958
    // albany: 14735
    // san juan: 11641
    let wbans = ["13958"/*, "23188"*/]; // TODO: WBAN #. Hard coded for now.
    // let mut location_temps: Vec<Vec<Vec<i16>>> = Vec::new();
    let mut location_temps: Vec<Vec<TempData>> = Vec::new();
    for wban in wbans {
        let data = download_data(year, wban, true);
        if let Err(error) = data {
            panic!("Could not download data: {}", error.to_string());
        }
        let mut data = data.unwrap();
        data.pop(); // Last record is always empty
        remove_invalid_entries(&mut data);
        let mut temperatures = extract_detailed_temps(&data);
        println!("Num temps: {}", temperatures.len());
        // remove_past_day(&mut temperatures, 7);
        // combine_like_temps(&mut temperatures);
        // plot_detailed_week_dist(&temperatures);
        // let daily_temps = extract_temps(&data, true);
        location_temps.push(temperatures);
    }
    calc_daily_average(&location_temps);
    // process_temps(&location_temps)
}

fn calc_daily_average(location_temps: &Vec<Vec<TempData>>) {
 for location in location_temps {
    // Sort all temps into bins
    let mut day_bins: Vec<Vec<&TempData>> = Vec::with_capacity(366);
    for i in 0..366 {
        day_bins.push(Vec::new()); 
    }
    for temp in location {
        let day_of_year = get_day_index_from_minutes(temp.minute_of_year);
        let bin = &mut day_bins[day_of_year];
        bin.push(&temp);
    } 
    
    // Calculate the average for every day
    let mut day_averages: Vec<Average> = Vec::with_capacity(366);
    for day in day_bins {
        let mut first_moment: i64 = 0;
        let mut second_moment: i64 = 0;
        for temp in day {
            // let x = temp.temp10 as i64 * temp.duration as i64;
            first_moment += temp.temp10 as i64 * temp.duration as i64;
            second_moment += temp.temp10 as i64 * temp.temp10 as i64 * temp.duration as i64;
        }
        let first_moment = first_moment as f64 / 14400.0; // Save all division until the end
        let second_moment = second_moment as f64 / 144000.0;
        let mean = first_moment;
        let variance = second_moment - (mean * mean);
        let standard_deviation = variance.sqrt();
        day_averages.push(Average { mean, standard_deviation });
        println!("Intermediate mean: {}, second_moment: {}, sd: {}, variance: {}", mean, second_moment, standard_deviation, variance);
    }

    // Calculate number of days used
    let weight = 1.0 / day_averages.len() as f64;

    // Calculate the total average and print it out
    let mut first_moment: f64 = 0.0;
    let mut second_moment: f64 = 0.0;
    for average in day_averages {
        let x = average.standard_deviation * weight;
        first_moment += x;
        second_moment += x * average.standard_deviation as f64;
    }
    let mean = first_moment;
    let variance = second_moment - (mean * mean);
    let standard_deviation = variance.sqrt();
    println!("Mean: {}, Standard deviation: {}", mean, standard_deviation);
 }   
}

fn get_day_index_from_minutes(minutes: u32) -> usize {
    return (minutes / 1440) as usize;
}

fn calc_weekly_average(location_temps: &Vec<TempData>) {

}

fn combine_like_temps (temperatures: &mut Vec<TempData>) {
    temperatures.sort_unstable_by(|a, b| a.temp10.cmp(&b.temp10));
    for i in 0..temperatures.len()-1 {
        if temperatures[i].temp10 == temperatures[i+1].temp10 {
            let new_duration = temperatures[i].duration + temperatures[i+1].duration;
            temperatures[i+1].duration = new_duration;
            temperatures[i].duration = 0;

        }
    }
    temperatures.retain(|t| t.duration != 0);
}

// Remove all entries past a particular day
fn remove_past_day (temperatures: &mut Vec<TempData>, day: u32) {
    let limit = 1440 * day;
    temperatures.retain(|temp| temp.minute_of_year < limit);
}

fn plot_detailed_year_temps(days: &Vec<TempData>) {
    let root_drawing_area = BitMapBackend::new("images/0.png", (1920, 1080))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();

    let mut chart: ChartContext<BitMapBackend, Cartesian2d<plotters::coord::types::RangedCoordi32, plotters::coord::types::RangedCoordf64>> = ChartBuilder::on(&root_drawing_area)
        .set_label_area_size(LabelAreaPosition::Bottom, 140)
        .set_label_area_size(LabelAreaPosition::Left, 240)
        .margin(50)
        // .caption("Temperatures Recorded at Camp Mabry Over a Year", ("sans-serif", 40))
        .build_cartesian_2d(0..527040, -15.0..50.0)
        .unwrap();

    chart.configure_mesh()
        .x_labels(15)
        .x_label_formatter(&|x| format!("{}", *x / 1440))
        .x_desc("Progress in Year (Days)")
        .y_labels(12)
        .y_desc("Temperature (°F)")
        .y_label_formatter(&|y| format!("{}", *y * (9.0/5.0) + 32.0))
        .axis_desc_style(("sans-serif", 70, &WHITE))
        .axis_style(&WHITE)
        .label_style(("sans-serif", 70, &WHITE))
        .draw().unwrap();

    chart.draw_series(
        days.iter().map(|t| Circle::new((t.minute_of_year as i32, t.temp10 as f64 / 10.0), 2, &WHITE))    
    )
    .unwrap()
    .label("Test");
}

fn plot_detailed_week_temps(days: &Vec<TempData>) {
    let root_drawing_area = BitMapBackend::new("images/0.png", (1920, 1080))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();

    let mut chart: ChartContext<BitMapBackend, Cartesian2d<plotters::coord::types::RangedCoordi32, plotters::coord::types::RangedCoordf64>> = ChartBuilder::on(&root_drawing_area)
        .set_label_area_size(LabelAreaPosition::Bottom, 140)
        .set_label_area_size(LabelAreaPosition::Left, 240)
        .margin(50)
        // .caption("Temperatures Recorded at Camp Mabry Over a Year", ("sans-serif", 40))
        .build_cartesian_2d(0..(7*1440), -15.0..50.0)
        .unwrap();

    chart.configure_mesh()
        .x_labels(7)
        .x_label_formatter(&|x| format!("{}", *x / 60))
        .x_desc("Progress in Week (Hours)")
        .y_labels(12)
        .y_desc("Temperature (°F)")
        .y_label_formatter(&|y| format!("{}", *y * (9.0/5.0) + 32.0))
        .axis_desc_style(("sans-serif", 70, &WHITE))
        .axis_style(&WHITE)
        .label_style(("sans-serif", 70, &WHITE))
        .draw().unwrap();

    chart.draw_series(
        days.iter().map(|t| Circle::new((t.minute_of_year as i32, t.temp10 as f64 / 10.0), 3, &WHITE))    
    )
    .unwrap()
    .label("Test");
}

fn plot_detailed_week_dist(days: &Vec<TempData>) {
    let root_drawing_area = BitMapBackend::new("images/0.png", (1920, 1080))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();

    let mut chart: ChartContext<BitMapBackend, Cartesian2d<plotters::coord::types::RangedCoordi32, plotters::coord::types::RangedCoordf64>> = ChartBuilder::on(&root_drawing_area)
        .set_label_area_size(LabelAreaPosition::Bottom, 140)
        .set_label_area_size(LabelAreaPosition::Left, 240)
        .margin(40)
        // .caption("Temperatures Recorded at Camp Mabry Over a Year", ("sans-serif", 40))
        .build_cartesian_2d(-150..150, 0f64..1000f64)
        .unwrap();

    chart.configure_mesh()
        .x_labels(12)
        .x_desc("Temperature (°F)")
        .x_label_formatter(&|x| format!("{:.1}", (*x as f32 / 10.0) * (9.0/5.0) + 32.0))
        .y_labels(10)
        .y_label_formatter(&|y| format!("{:.3}", *y as f32 / (7*1440) as f32))
        .y_desc("Probability")
        .axis_desc_style(("sans-serif", 70, &WHITE))
        .axis_style(&WHITE)
        .label_style(("sans-serif", 50, &WHITE))
        .draw().unwrap();

        chart.draw_series(
            Histogram::vertical(&chart)
                .style(RED.mix(0.8).filled())
                .data(days.iter().map(|t| (t.temp10 as i32, t.duration as f64))),
        ).unwrap();
}

fn plot_detailed_day_temps(days: &Vec<TempData>) {
    let root_drawing_area = BitMapBackend::new("images/0.png", (1920, 1080))
        .into_drawing_area();

    root_drawing_area.fill(&BLACK).unwrap();

    let mut chart: ChartContext<BitMapBackend, Cartesian2d<plotters::coord::types::RangedCoordi32, plotters::coord::types::RangedCoordf64>> = ChartBuilder::on(&root_drawing_area)
        .set_label_area_size(LabelAreaPosition::Bottom, 140)
        .set_label_area_size(LabelAreaPosition::Left, 240)
        .margin(50)
        // .caption("Temperatures Recorded at Camp Mabry Over a Year", ("sans-serif", 40))
        .build_cartesian_2d(0..(1440), -15.0..50.0)
        .unwrap();

    chart.configure_mesh()
        .x_labels(24)
        .x_label_formatter(&|x| format!("{}", *x / 60))
        .x_desc("Progress in Week (Hours)")
        .y_labels(12)
        .y_desc("Temperature (°F)")
        .y_label_formatter(&|y| format!("{}", *y * (9.0/5.0) + 32.0))
        .axis_desc_style(("sans-serif", 70, &WHITE))
        .axis_style(&WHITE)
        .label_style(("sans-serif", 70, &WHITE))
        .draw().unwrap();

    chart.draw_series(
        days.iter().map(|t| Circle::new((t.minute_of_year as i32, t.temp10 as f64 / 10.0), 3, &WHITE))    
    )
    .unwrap()
    .label("Test");
}

// Downloads data from the NOAA for a specific WBAN. Requires several discrete
// steps:
// - Finding the exact filepath to get the data
// - Downloading the data
// - Decompressing the data
fn download_data(year: &str, wban: &str, save_output: bool) -> Result<Vec<String>, ureq::Error> {
    // FIXME: We use the NOAA website rather than the API. I'd prefer the API,
    // but it's a pain in the rear. A pain for a later date.
    let url: String = format!("https://www1.ncdc.noaa.gov/pub/data/noaa/{}/", year);
    let body = ureq::get(&url)
        .call()?
        .into_string()?;
    // TODO: Add a cached file with these results so we do not have to wait for
    // the website every time. Or, possibly even better, add a cached file which contains
    // stations that have been visited before.
    let mut start_index = 11 + body.find("</td></tr>").expect("Could not find start of table");
    let zip_file: String;
    loop {
        let wban_start = start_index + 24;
        let wban_end = start_index + 29;
        let captured_wban = body.get(wban_start..wban_end).unwrap(); // Assume WBAN# exists in list
        if wban.eq(captured_wban) {
            zip_file = body.get(start_index+17..start_index+37).unwrap().to_string();
            break;
        }
        start_index += 157;
    }
    
    // We have the data url to download now
    let url: String = format!("https://www1.ncdc.noaa.gov/pub/data/noaa/{year}/{file}", year = year, file = zip_file);
    let response = ureq::get(&url).call()?; // TODO: This could be several MB of data. Be careful
    let len: usize = response.header("Content-Length")
    .unwrap()
    .parse().unwrap();
    let mut bytes = Vec::<u8>::with_capacity(len);
    response.into_reader().take(10_000_000).read_to_end(&mut bytes)?;
    
    let mut decoded = String::new();
    let mut decoder = GzDecoder::new(bytes.as_slice());
    decoder.read_to_string(&mut decoded)?;
    let save_path = format!("./data/{wban}-{year}", wban=wban, year=year);
    if save_output {
        fs::write(save_path, decoded.clone())?; // TODO: This write may be dangerous if concurrent requests happen
    }
    let lines = decoded.split('\n').map(str::to_string).collect();
    return Ok(lines);
}

// Remove invalid entries from the data. Currently, valid entries are entries
// which have passed all NOAA quality control checks and come from official NOAA
// sources.
fn remove_invalid_entries(data: &mut Vec<String>) {    
    data.retain(
        |line|
        (line.get(56..59).unwrap().eq("V03") || line.get(56..59).unwrap().eq("V02")) && line.get(92..93).unwrap().eq("5")
    );
    // for i in 0..data.len() {
    //     println!("{}", data[i]);
    // }

}

// Do the bulk of the handling of the data lmao
fn extract_temps(data: &Vec<String>, save_output: bool) -> Vec<Vec<i16>> {
    let year = usize::from_str_radix(data[0].get(15..19).unwrap(), 10).unwrap(); 
    let leap_year = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_year: usize = if leap_year { 366 } else { 365 };
    let mut daily_temps: Vec<Vec<i16>> = Vec::with_capacity(days_in_year);
    for _i in 0..days_in_year {
        let day_temps: Vec<i16> = Vec::with_capacity(24);
        daily_temps.push(day_temps);
    }
    // let mut daily_temps: Vec<&mut Vec<u16>> = Vec::with_capacity(days_in_year);
    for line in data {
        let day_of_year = get_day_of_year(line.get(19..23).unwrap(), leap_year);
        // println!("Temp: {}", line.get(87..92).unwrap());
        let temperature = i16::from_str_radix(line.get(87..92).unwrap(), 10).unwrap();
        let day = &mut daily_temps[day_of_year-1];
        day.push(temperature);        
    }
    // We now have an array of 
    return daily_temps;
}

fn extract_detailed_temps(raw_data: &Vec<String>) -> Vec<TempData> {
    let year = usize::from_str_radix(raw_data[0].get(15..19).unwrap(), 10).unwrap(); 
    let leap_year = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let mut temperatures = Vec::<TempData>::new();
    // let mut daily_temps: Vec<&mut Vec<u16>> = Vec::with_capacity(days_in_year);
    for i in 0..raw_data.len()-1 {
        let line = &raw_data[i];
        let day_of_year = get_day_of_year(line.get(19..23).unwrap(), leap_year);
        let temperature = i16::from_str_radix(line.get(87..92).unwrap(), 10).unwrap();
        let hours = u32::from_str_radix(line.get(23..25).unwrap(), 10).unwrap();
        let minutes = u32::from_str_radix(line.get(25..27).unwrap(), 10).unwrap();
        let minute_of_year = (day_of_year as u32 - 1) * 1440 + hours * 60 + minutes;
        
        let next_line = &raw_data[i+1];
        let next_day_of_year = get_day_of_year(next_line.get(19..23).unwrap(), leap_year);
        let next_hours = u32::from_str_radix(next_line.get(23..25).unwrap(), 10).unwrap();
        let next_minutes = u32::from_str_radix(next_line.get(25..27).unwrap(), 10).unwrap();
        let next_minute_of_year = (next_day_of_year as u32 - 1) * 1440 + next_hours * 60 + next_minutes;
        // println!("Day: {}, Hours: {}, Minutes: {}, Minutes of year: {}", day_of_year, hours, minutes, minute_of_year);
        temperatures.push(TempData{temp10: temperature, minute_of_year, duration: (next_minute_of_year - minute_of_year) as u16});        
    }
    let line = &raw_data[raw_data.len()-1];
    let day_of_year = get_day_of_year(line.get(19..23).unwrap(), leap_year);
    let temperature = i16::from_str_radix(line.get(87..92).unwrap(), 10).unwrap();
    let hours = u32::from_str_radix(line.get(23..25).unwrap(), 10).unwrap();
    let minutes = u32::from_str_radix(line.get(25..27).unwrap(), 10).unwrap();
    let minute_of_year = (day_of_year as u32 - 1) * 1440 + hours * 60 + minutes;
    temperatures.push(TempData{temp10: temperature, minute_of_year, duration: 1440u16 - (temperatures[temperatures.len() - 1].minute_of_year as u16 % 1440u16)});
    // We now have an array of 
    return temperatures;
}

struct TempData {
    temp10: i16,
    duration: u16, // duration in minutes until next sample. Will be used to calculate probability.
    minute_of_year: u32
}

// Gets the body from an HTTP request to a website
// async fn get_http_body(url: &str) -> Result<String, reqwest::Error> {
//     return reqwest::get(url)
//         .await?
//         .text()
//         .await
// }

// Calculate what day of the year it is given a particular date in MMDD
// format. This will be used as an index into the array of dates
fn get_day_of_year(date: &str, leap_year: bool) -> usize {
    let month_str = &date[..2];
    let day_str = &date[2..4];
    let month_completion: [usize; 12] = if leap_year {
        [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335]
    } else {
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334]
    };
    let month = usize::from_str_radix(month_str, 10).unwrap(); 
    let day = usize::from_str_radix(day_str, 10).unwrap(); 
    return day + month_completion[month - 1];
}

// For each temperature:
//  - Calculate the mean and the standard deviation of each day
//  - Calculate the mean and the standard deviation of those means
//  - Store this
// For all temperatures:
//  - Compare the daily means and standard deviations and determine which is
//    weirder 
//  - Generate graphs which show the change over time
// TODO: Change temperatures to be a float of f64s. No need to waste time
// converting more than once
fn process_temps(locations: &Vec<Vec<Vec<i16>>>) {
    // This has the following access patern:
    // - location_temps[location_index][day_of_year][temperature_index]
    let mut location_averages: Vec<Vec<Average>> = Vec::with_capacity(locations.len());
    for days_of_week in locations {
        let mut daily_average: Vec<Average> = Vec::with_capacity(days_of_week.len());
        for temps in days_of_week {
            let num_temps_for_today = temps.len();
            let mut first_moment: i64 = 0;
            let mut second_moment: i64 = 0;
            for &temp in temps {
                let t = temp as i64;
                first_moment += t;
                second_moment += t * t;
            }
            // Single divide for all recorded values because:
            // 1. Temps from the dataset are stored *10
            // 2. moments should be divided by their weights
            let first_moment: f64 = first_moment as f64 / (num_temps_for_today as f64 * 10.0);
            let second_moment: f64 = second_moment as f64 / (num_temps_for_today as f64 * 100.0);
            let variance = second_moment - (first_moment * first_moment);
            let standard_deviation = variance.sqrt();
            daily_average.push(Average{mean: first_moment, standard_deviation});
        }
        location_averages.push(daily_average);
    }

    let mut city_averages: Vec<(Average, Average)> = Vec::new();
    for location in location_averages {
        let mut valid_days = 0;
        let mut mean_m1 = 0.0;
        let mut mean_m2 = 0.0;
        let mut sd_m1 = 0.0;
        let mut sd_m2 = 0.0;
        for average in location {
            if average.mean.is_nan() {
                continue;
            }
            valid_days += 1;
            mean_m1 += average.mean;
            mean_m2 += average.mean * average.mean;
            sd_m1 += average.standard_deviation;
            sd_m2 += average.standard_deviation * average.standard_deviation;
            // println!("Mean: {}, SD: {}", average.mean, average.standard_deviation);
        }
        mean_m1 /= valid_days as f64;
        mean_m2 /= valid_days as f64;
        sd_m1 /= valid_days as f64;
        sd_m2 /= valid_days as f64;
        let mean_sd = (mean_m2 - (mean_m1 * mean_m1)).sqrt();
        let average_mean = Average{mean: mean_m1, standard_deviation: mean_sd};
        let sd_sd = (sd_m2 - (sd_m1 * sd_m1)).sqrt();
        let average_sd = Average{mean: sd_m1, standard_deviation: sd_sd};
        city_averages.push((average_mean, average_sd));
    }

    for location in city_averages {
        let (mean, sd) = location;
        println!("City mean: {} +- {}\nCity SD: {} +- {}", 
            mean.mean, mean.standard_deviation, sd.mean, sd.standard_deviation);
    }
}   

struct Average {
    mean: f64,
    standard_deviation: f64,
}
  /*
  fn day_of_year(date: &str) -> usize {
    let year_str = &date[0..4];
    let month_str = &date[4..6];
    let day_str = &date[6..8];
    let mut months = vec![31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30];
    let year = usize::from_str_radix(year_str, 10).unwrap(); 
    if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { // Leap year check
        months[1] = 29;
    }
    let month = usize::from_str_radix(month_str, 10).unwrap(); 
    let day = usize::from_str_radix(day_str, 10).unwrap(); 
    
    return day + months[month - 1];
  }
   */