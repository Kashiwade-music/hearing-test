use crossterm::{execute, queue};
use rodio::source::Source;
use rodio::{Decoder, OutputStream, Sink};
use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use crossterm::{
    cursor,
    event::{read, Event},
    style::{self, Stylize},
    terminal, ExecutableCommand, QueueableCommand, Result,
};
use std::io::{stdout, BufRead, BufReader, Write};

mod csv;
mod plot;
mod sinewave;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Point {
    memo: String,
    test_freq: Vec<f32>,
}

fn main() {
    // init
    let mut stdout = stdout();
    let config = load_yaml_config(Path::new("config.yaml"));
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let mut result: BTreeMap<String, BTreeMap<i32, f32>> = BTreeMap::new();

    //-------------------
    // let sink = Sink::try_new(&stream_handle).unwrap();
    // //play 02 CD Track 02.wav
    // let file = BufReader::new(File::open("02 CD Track 02.wav").unwrap());
    // let source = Decoder::new(file).unwrap();
    // sink.append(source);
    //-------------------

    queue!(
        stdout,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
        style::PrintStyledContent("Welcome to the sound test!".bold()),
        cursor::MoveToNextLine(1),
        style::PrintStyledContent("Your config memo is: ".green()),
        style::Print(format!("{:?}", config.memo)),
        cursor::MoveToNextLine(1),
        style::PrintStyledContent("Your test frequencies are: ".green()),
        style::Print(format!("{:?}", config.test_freq)),
        cursor::MoveToNextLine(2),
        style::SetBackgroundColor(style::Color::Red),
        style::SetForegroundColor(style::Color::White),
        style::PrintStyledContent("IMPORTANT:".bold().white()),
        style::SetBackgroundColor(style::Color::Reset),
        style::SetForegroundColor(style::Color::Reset),
        style::Print(" We will play a sin wave with a peak of ".to_string()),
        style::PrintStyledContent("-24dBFS(max. -19.7LUFS)".bold()),
        style::Print(". Make sure that the volume is not too loud.".to_string()),
        cursor::MoveToNextLine(2),
        style::Print("Press Enter key to start the test.".to_string()),
    )
    .unwrap();
    stdout.flush().unwrap();
    loop {
        if let Event::Key(key) = read().unwrap() {
            if key.code == crossterm::event::KeyCode::Enter {
                break;
            }
        }
    }

    queue!(stdout, terminal::Clear(terminal::ClearType::All),).unwrap();
    stdout.flush().unwrap();

    // play
    // iterate over the test_freq
    for channel in 0..2 {
        result.insert(
            if channel == 0 {
                "L".to_string()
            } else {
                "R".to_string()
            },
            BTreeMap::new(),
        );
        for freq in &config.test_freq {
            queue!(
            stdout,
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::CurrentLine),
            cursor::MoveToNextLine(1),
            terminal::Clear(terminal::ClearType::CurrentLine),
            cursor::MoveTo(0, 0),
            style::PrintStyledContent("Press the up/down keys to adjust the volume and press Enter the first time you hear nothing.".bold()),
            cursor::MoveToNextLine(1),
            style::PrintStyledContent("Current frequency: ".green()),
            style::Print(format!("{:?} Hz", freq)),
            style::PrintStyledContent("   LR: ".green()),
            style::Print(format!("{:?}", if channel == 0 { "L" } else { "R" })),
            cursor::MoveToNextLine(1),
            style::Print("If audio signal channel is swapped, press 'r' to reset.".to_string()),
            cursor::MoveToNextLine(1),
        )
        .unwrap();
            stdout.flush().unwrap();

            // reset sink
            let mut sink = Sink::try_new(&stream_handle).unwrap();
            let mut source = sinewave::SineWave::new(*freq, 0.4, 0.4, channel);
            sink.set_volume(db_to_float(-24.0) as f32);
            sink.append(source);

            // Loop until enter is pressed
            // Increase volume by 0.5db when the up arrow key is pressed
            // Decrease volume by 0.5db when the down arrow key is pressed
            loop {
                if let Ok(Event::Key(event)) = read() {
                    match event.code {
                        crossterm::event::KeyCode::Up => {
                            sink.set_volume(sink.volume() * db_to_float(2.0) as f32);
                        }
                        crossterm::event::KeyCode::Down => {
                            sink.set_volume(sink.volume() * db_to_float(-2.0) as f32);
                        }
                        crossterm::event::KeyCode::Char('r') => {
                            sink.stop();
                            sink = Sink::try_new(&stream_handle).unwrap();
                            source = sinewave::SineWave::new(*freq, 0.4, 0.4, channel);
                            sink.set_volume(db_to_float(-24.0) as f32);
                            sink.append(source);
                        }
                        crossterm::event::KeyCode::Enter => {
                            sink.stop();
                            // print volume
                            queue!(
                                stdout,
                                cursor::MoveToNextLine(1),
                                terminal::Clear(terminal::ClearType::CurrentLine),
                                style::PrintStyledContent(
                                    format!("Result of {:?} Hz -> Volume: ", *freq as i32).green()
                                ),
                                style::Print(format!(
                                    "{:.3} dB",
                                    float_to_db(sink.volume() as f64)
                                )),
                            )
                            .unwrap();
                            stdout.flush().unwrap();

                            // write result
                            result
                                .get_mut(if channel == 0 { "L" } else { "R" })
                                .unwrap()
                                .insert(*freq as i32, float_to_db(sink.volume() as f64) as f32);

                            // wait 0.5sec
                            std::thread::sleep(Duration::from_millis(800));
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // print result
    queue!(
        stdout,
        cursor::MoveToNextLine(1),
        terminal::Clear(terminal::ClearType::CurrentLine),
        style::PrintStyledContent("Result: ".green()),
        cursor::MoveToNextLine(1),
        style::PrintStyledContent("L: ".green()),
        style::Print(format!("{:?}", result.get("L").unwrap())),
        cursor::MoveToNextLine(1),
        style::PrintStyledContent("R: ".green()),
        style::Print(format!("{:?}", result.get("R").unwrap())),
    )
    .unwrap();
    stdout.flush().unwrap();

    let now_date = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
    // plot
    plot::plot_audiogram(result.clone(), "./result", &now_date);

    csv::save_to_csv(result, "./result", &now_date)
}

fn load_yaml_config(path: &Path) -> Point {
    // loaf yaml file and load as string
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    let mut yaml_str = String::new();
    // load lines with LF
    for line in reader.lines() {
        yaml_str.push_str(&line.unwrap());
        yaml_str.push_str(&"\n");
    }

    // parse yaml string
    let points: Point = serde_yaml::from_str(&yaml_str).unwrap();
    return points;
}

fn float_to_db(float: f64) -> f64 {
    return 20.0 * float.log10();
}

fn db_to_float(db: f64) -> f64 {
    return 10.0_f64.powf(db / 20.0);
}
