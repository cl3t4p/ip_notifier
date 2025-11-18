
use std::path::Path;
use std::time::Duration;
use std::{fs, thread};
use reqwest::blocking::{Client, Response};
use reqwest::header::CONTENT_TYPE;
use reqwest::{Error};

const OLD_IP_FILE_NAME : &str  = "config/old_ip.tmp";
const WEBHOOK_FILE_NAME : &str = "config/webhook.json";
const CONFIG_FILE_NAME : &str = "config/config.toml";


fn main() {
    //Read config.toml or create it if it does not exist
    if !Path::new("config").exists() {
        fs::create_dir("config").expect("Unable to create config directory");
    }

    if !Path::new(CONFIG_FILE_NAME).exists() {
        fs::write(CONFIG_FILE_NAME, "webhook = \"\"\nwait_seconds = 60")
            .expect("Unable to create config.toml");
        println!("config.toml created. Please fill in the token and webhook fields.");
        return;
    }
    // Read the config file
    let config_content = fs::read_to_string(CONFIG_FILE_NAME)
        .expect("Unable to read config.toml");
    let config: toml::Value = toml::from_str(&config_content).unwrap();
    let webhook = config.get("webhook").and_then(|v| v.as_str()).unwrap_or("");
    let wait_seconds = config.get("wait_seconds").and_then(|v| v.as_integer()).unwrap_or(60) as u64;
    let ip_grab_url = config.get("ip_grab_url").and_then(|v| v.as_str()).unwrap_or("https://api.ipify.org");
    let blacklist_words = config.get("blacklist_words").and_then(|v| v.as_array()).unwrap_or(&vec![])
        .iter()
        .map(|val| val.as_str().map(|s| s.to_string()))
        .collect::<Vec<Option<String>>>();



    if webhook.is_empty() {
        println!("Webhook URL is not set in config.toml. Please set it and try again.");
        return;
        
    }

    // Initialize the logger
    let mut log = paris::Logger::new();
    log.info("Starting IP change notifier...");



    // Check if the old_ip.txt file exists
    let mut old_ip : String;
    if Path::new(OLD_IP_FILE_NAME).exists() {
        old_ip =  fs::read_to_string(OLD_IP_FILE_NAME).unwrap();
    }else{
        loop {
            match get_new_ip(ip_grab_url) {
                Ok(response) => match response.text() {
                    Ok(ip) => {
                        old_ip = ip;
                        break;
                    }
                    Err(_) => {
                        log.error("Failed to get IP, retrying...");
                        thread::sleep(Duration::from_secs(2));
                    }
                },
                Err(_) => {
                    log.error("Failed to get IP, retrying...");
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }
        // If the file does not exist, create it with the current IP
        fs::write(OLD_IP_FILE_NAME, old_ip.clone())
            .expect("Unable to write to old_ip.txt");
        send_ip(old_ip.clone(),webhook).expect("Error sending initial IP");
    }


    ctrlc::set_handler(move || {
        println!("Ctrl-C received, exiting...");
        //Write the current IP to old_ip.txt before exiting
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");


    loop {
        // Log the current IP address
        log.info("Checking for IP changes...");
        let temp_ip_res = get_new_ip(ip_grab_url);


        // If there is an error getting the current IP, log it
        if temp_ip_res.is_err() {
            log.error("Error getting the current IP address.");
        }else{
            let temp_ip = temp_ip_res.unwrap().text().unwrap();
            // Check blacklist using regex patterns
            let mut blacklist_match = false;
            // Iterate through blacklist words and check if current IP matches any pattern
            for word in blacklist_words.iter() {
                let pattern = match word {
                    Some(s) if !s.is_empty() => s,
                    _ => continue,
                };
                match regex::Regex::new(pattern) {
                    Ok(re) => {
                        if re.is_match(&temp_ip) {
                            log.error(format!("Current IP {} matches blacklist pattern '{}'. Skipping.", temp_ip, pattern).as_str());
                            blacklist_match = true;
                            break;
                        }
                    }
                    Err(err) => {
                        log.error(format!("Invalid regex pattern '{}': {}. Skipping pattern.", pattern, err).as_str());
                    }
                }
            }
            if blacklist_match {
                thread::sleep(Duration::from_secs(wait_seconds));
                continue;
            }
            log.info(format!("Current IP: {}", temp_ip).as_str());

            if temp_ip != old_ip.clone() {
                match send_ip(temp_ip.clone(),webhook) {
                    Ok(ok) => {
                        if ok.status().is_success() {
                            log.info("IP sent successfully.");
                        } else {
                            log.error(format!("Failed to send IP: {}", ok.status()).as_str());
                        }
                    },
                    Err(err) => {
                        log.error(format!("Error sending IP: {}", err).as_str());
                    },
                }
                fs::write(OLD_IP_FILE_NAME, temp_ip.clone())
                    .expect("Unable to write to old_ip.txt");
                log.info(format!("IP changed to: {}", temp_ip).as_str());
                old_ip = temp_ip;
            }
        }

        thread::sleep(Duration::from_secs(wait_seconds));
    }

}


fn send_ip(ip : String,webhook : &str) -> Result<Response, Error> {

    // Read the webhook.json file and replace #ip# with the current IP
    if !Path::new(WEBHOOK_FILE_NAME).exists() {
        let contents = r#"{
    "content" : "IP has changed to #ip#",
    "username" : "IP Notifier"
}"#.to_string();
        fs::write(WEBHOOK_FILE_NAME, contents).unwrap();
    }
    let mut webhook_json = fs::read_to_string(WEBHOOK_FILE_NAME)
        .expect("Reading webhook.json");

    webhook_json = webhook_json.replace("#ip#", ip.as_str());

    let client = Client::new();
    return client
        .post(webhook)
        .header(CONTENT_TYPE, "application/json")
        .body(webhook_json)
        .send();
}

fn get_new_ip(ip_grab_url: &str) -> Result<Response,Error>{

    let client = Client::new();
    return client.get(ip_grab_url)
    .send();
}