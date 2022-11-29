use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn check_exist_csv(dir_path: &str) -> bool {
    let path = Path::new(dir_path);
    if !path.exists() {
        fs::create_dir(dir_path).unwrap();
    }

    // check exist result_[0-9]{2}.csv
    let mut exist = false;
    for entry in fs::read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let file_name = path.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with("result_") {
            exist = true;
            break;
        }
    }
    exist
}

// csv file format is like this:
// date, LR, 62, 125, 250, 500, 1000, 1500, 2000, 3000, 4000, 6000, 8000, 10000, 12000
// 20221128_205550, L, 24.0, 32.0, 10.0, 20.0, 30.0, 40.0, 45.0, 65.0, 23.0, 87.0, 13.0, 23.0, 56.0
// 20221128_205550, R, 24.0, 32.0, 10.0, 20.0, 30.0, 40.0, 45.0, 65.0, 23.0, 87.0, 13.0, 23.0, 56.0
// 20221128_205550, L, 24.0, 32.0, 10.0, 20.0, 30.0, 40.0, 45.0, 65.0, 23.0, 87.0, 13.0, 23.0, 56.0
// 20221128_205550, R, 24.0, 32.0, 10.0, 20.0, 30.0, 40.0, 45.0, 65.0, 23.0, 87.0, 13.0, 23.0, 56.0
pub fn load_csv(dir_path: &str) -> BTreeMap<String, BTreeMap<String, BTreeMap<i32, f32>>> {
    let mut result = BTreeMap::new();

    // at dir_path, there are result_01.csv or result_02.csv
    // load latest csv file
    let path = fs::read_dir(dir_path)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().unwrap().is_file())
        .filter(|entry| entry.file_name().to_str().unwrap().contains("result_"))
        .max_by_key(|entry| entry.file_name())
        .unwrap()
        .path();

    let mut reader = csv::Reader::from_path(path).unwrap();
    let title_row = reader.headers().unwrap().to_owned();
    println!("{:?}", title_row);
    for row in reader.records() {
        let row = row.unwrap();
        let date = row.get(0).unwrap().to_string();
        let lr = row.get(1).unwrap().to_string();
        let mut data = BTreeMap::new();
        for (i, value) in row.iter().enumerate() {
            if i > 1 {
                println!("{}: {}", title_row.get(i).unwrap(), value);
                let freq = title_row.get(i).unwrap().parse::<i32>().unwrap();
                let volume = value.parse::<f32>().unwrap();
                data.insert(freq, volume);
            }
        }
        result
            .entry(date)
            .or_insert(BTreeMap::new())
            .insert(lr, data);
    }
    result
}

pub fn save_to_csv(result: BTreeMap<String, BTreeMap<i32, f32>>, dir_path: &str, now_date: &str) {
    // if dir_path is not exist, create dir
    if !fs::metadata(dir_path).is_ok() {
        fs::create_dir(dir_path).unwrap();
    }

    // load current csv file
    if check_exist_csv(dir_path) {
        let mut csv = load_csv(dir_path);
        let mut is_same = true;
        let first_data = csv.values().next().unwrap().values().next().unwrap();
        for (freq, _) in result.values().next().unwrap() {
            if !first_data.contains_key(freq) {
                is_same = false;
                break;
            }
        }
        for (freq, _) in first_data {
            if !result.values().next().unwrap().contains_key(freq) {
                is_same = false;
                break;
            }
        }

        if !is_same {
            let mut max = 0;
            for entry in fs::read_dir(dir_path)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().unwrap().is_file())
                .filter(|entry| entry.file_name().to_str().unwrap().contains("result_"))
            {
                let file_name = entry.file_name().to_str().unwrap().to_string();
                let num = file_name[7..file_name.len() - 4].parse::<i32>().unwrap();
                if num > max {
                    max = num;
                }
            }
            let mut path = dir_path.to_string();
            path.push_str("\\result_");
            path.push_str(format!("{:02}", max + 1).as_str());
            path.push_str(".csv");
            let mut writer = csv::Writer::from_path(path).unwrap();
            let mut culumn = vec!["date".to_string(), "LR".to_string()];
            for freq in result.get("L").unwrap().keys() {
                culumn.push(freq.to_string());
            }
            writer.write_record(&culumn).unwrap();
            for (lr, data) in result.iter() {
                let mut record = vec![now_date.to_string(), lr.to_string()];
                for freq in data.keys() {
                    record.push(data.get(freq).unwrap().to_string());
                }
                writer.write_record(&record).unwrap();
            }
            writer.flush().unwrap();
        } else {
            //write to csv file
            // get latest csv file path
            let path = fs::read_dir(dir_path)
                .unwrap()
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().unwrap().is_file())
                .filter(|entry| entry.file_name().to_str().unwrap().contains("result_"))
                .max_by_key(|entry| entry.file_name())
                .unwrap()
                .path();

            // write to csv file
            csv.insert(now_date.to_string(), result);
            let mut writer = csv::Writer::from_path(path).unwrap();
            let mut culumn = vec!["date".to_string(), "LR".to_string()];
            for freq in csv.values().next().unwrap().values().next().unwrap().keys() {
                culumn.push(freq.to_string());
            }
            writer.write_record(&culumn).unwrap();
            for (date, lr) in csv {
                for (lr, data) in lr {
                    let mut row = vec![date.to_string(), lr.to_string()];
                    for freq in culumn.iter().skip(2) {
                        row.push(data.get(&freq.parse::<i32>().unwrap()).unwrap().to_string());
                    }
                    writer.write_record(&row).unwrap();
                }
            }
            writer.flush().unwrap();
        }
    } else {
        //write to csv file
        let mut path = dir_path.to_string();
        path.push_str("\\result_01.csv");
        let mut writer = csv::Writer::from_path(path).unwrap();
        let mut culumn = vec!["date".to_string(), "LR".to_string()];
        for freq in result.get("L").unwrap().keys() {
            culumn.push(freq.to_string());
        }
        writer.write_record(&culumn).unwrap();
        for (lr, data) in result.iter() {
            let mut record = vec![now_date.to_string(), lr.to_string()];
            for freq in data.keys() {
                record.push(data.get(freq).unwrap().to_string());
            }
            writer.write_record(&record).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_exist_csv() {
        let dir_path = "./result";
        let exist = check_exist_csv(dir_path);
        assert_eq!(exist, false);
    }

    #[test]
    fn test_load_csv() {
        let dir_path = "./result";
        let result = load_csv(dir_path);
        println!("{:?}", result);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_save_to_csv() {
        let mut result = BTreeMap::new();
        let mut data = BTreeMap::new();
        data.insert(100, 0.1);
        data.insert(200, 0.2);
        data.insert(300, 0.3);
        data.insert(400, 0.4);
        data.insert(500, 0.5);
        data.insert(600, 0.6);
        data.insert(700, 0.7);
        data.insert(800, 0.8);
        data.insert(900, 0.9);
        data.insert(1000, 1.0);
        result.insert("L".to_string(), data);
        let mut data = BTreeMap::new();
        data.insert(100, 0.1);
        data.insert(200, 0.2);
        data.insert(300, 0.3);
        data.insert(400, 0.4);
        data.insert(500, 0.5);
        data.insert(600, 0.6);
        data.insert(700, 0.7);
        data.insert(800, 0.8);
        data.insert(900, 0.9);
        data.insert(1000, 1.0);
        result.insert("R".to_string(), data);
        let dir_path = "./result";
        let now_date = "20221128_205550";
        save_to_csv(result, dir_path, now_date);
    }

    #[test]
    fn test_all() {
        let dir_path = "./result";

        // delete all csv file, if exist
        for entry in fs::read_dir(dir_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().unwrap().is_file())
            .filter(|entry| entry.file_name().to_str().unwrap().contains("result_"))
        {
            fs::remove_file(entry.path()).unwrap();
        }

        assert_eq!(check_exist_csv(dir_path), false);

        let now_date = "20221128_205550";
        let mut result = BTreeMap::new();
        let mut data = BTreeMap::new();
        data.insert(100, 0.1);
        data.insert(200, 0.2);
        data.insert(300, 0.3);
        data.insert(400, 0.4);
        data.insert(500, 0.5);
        data.insert(600, 0.6);
        data.insert(700, 0.7);
        data.insert(800, 0.8);
        data.insert(900, 0.9);
        data.insert(1000, 1.0);
        result.insert("L".to_string(), data);
        let mut data = BTreeMap::new();
        data.insert(100, 0.1);
        data.insert(200, 0.2);
        data.insert(300, 0.3);
        data.insert(400, 0.4);
        data.insert(500, 0.5);
        data.insert(600, 0.6);
        data.insert(700, 0.7);
        data.insert(800, 0.8);
        data.insert(900, 0.9);
        data.insert(1000, 1.0);
        result.insert("R".to_string(), data);
        save_to_csv(result.clone(), dir_path, now_date);

        assert_eq!(check_exist_csv(dir_path), true);
        assert_eq!(load_csv(dir_path).len(), 1);

        save_to_csv(result.clone(), dir_path, "20221129_200000");
        assert_eq!(load_csv(dir_path).len(), 2);

        let mut result = BTreeMap::new();
        let mut data = BTreeMap::new();
        data.insert(100, 0.1);
        data.insert(300, 0.3);
        data.insert(400, 0.4);
        data.insert(500, 0.5);
        data.insert(600, 0.6);
        data.insert(700, 0.7);
        data.insert(800, 0.8);
        data.insert(900, 0.9);
        data.insert(1000, 1.0);
        result.insert("L".to_string(), data);
        let mut data = BTreeMap::new();
        data.insert(100, 0.1);
        data.insert(300, 0.3);
        data.insert(400, 0.4);
        data.insert(500, 0.5);
        data.insert(600, 0.6);
        data.insert(700, 0.7);
        data.insert(800, 0.8);
        data.insert(900, 0.9);
        data.insert(1000, 1.0);
        result.insert("R".to_string(), data);

        save_to_csv(result.clone(), dir_path, "20221130_200000");
        assert_eq!(load_csv(dir_path).len(), 1);
    }
}
