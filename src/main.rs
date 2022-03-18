use thirtyfour::prelude::*;
use thirtyfour::common::capabilities::chrome::ChromeCapabilities;
use tokio;
use serde::{Serialize, Deserialize};
use serde_json;
use std::fs;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Opts {
    profile: String,
    base_url: String,
    username: String,
    password: String,
    csv_file_path: String,
    error_message: String,
    driver_args: Vec<String>,
}

#[tokio::main]
async fn main() -> WebDriverResult<()> {
    // loads options file
    let opts: Opts = load_opts();

    // loads chrome options
    let caps: ChromeCapabilities = load_caps(opts.clone());
    
    // starts new webdriver
    let driver: WebDriver = WebDriver::new("http://localhost:4444", &caps).await?;
    
    // navigates to url
    driver.get(opts.clone().base_url).await?;

    // checks if user is logged in
    let is_logged_in = !is_logged_in(&driver, opts.clone()).await?;

    // if user is not logged in, logs user in
    if !is_logged_in {
        println!("not logged in!\n");
        login(&driver, opts.clone()).await?;
    } else {
        println!("already logged in!\n")
    }
    
    // switches to correct tab
    switch_tab(&driver).await?;

    Ok(())
}

// loads options file
fn load_opts() -> Opts {
    println!("loading options file...\n");

    let file_str: String = fs::read_to_string("data/options.json").expect("unable to read file!");
    
    serde_json::from_str(&file_str).expect("unable to parse file!")
}

// loads chrome options
fn load_caps(opts: Opts) -> ChromeCapabilities {
    println!("loading chrome options...\n");

    let mut caps: ChromeCapabilities = DesiredCapabilities::chrome();
    for opt in opts.driver_args {
        caps.add_chrome_arg(&opt).expect("successfully added argument {:?}");
    }

    caps
}

// checks if user is logged in
// if not logged in, the website redirects to an auth0 login page
async fn is_logged_in(driver: &WebDriver, opts: Opts) -> WebDriverResult<bool> {
    let url = driver.current_url().await?;

    Ok(!opts.base_url.contains(&url))
}

// logs user in
async fn login(driver: &WebDriver, opts: Opts) -> WebDriverResult<()> {
    println!("logging in...\n");

    let login_form: WebElement = driver.find_element(By::Tag("form")).await?;

    let username_input: WebElement = login_form.find_element(By::Id("username")).await?;
    let password_input: WebElement = login_form.find_element(By::Id("password")).await?;
    let form_submit_btn: WebElement = login_form.find_element(By::Tag("button[type=\"submit\"]")).await?;
    
    write(username_input, opts.username).await?;
    write(password_input, opts.password).await?;

    form_submit_btn.click().await?;

    Ok(())
}

// switches to correct tab
async fn switch_tab(driver: &WebDriver) -> WebDriverResult<()> {
    println!("switching to correct tab...\n");

    let tab: WebElement = driver.query(By::XPath("//span[contains(text(), \"Sent and pending\")]/..")).first().await?;
    tab.click().await?;

    Ok(())
}

// clears input and writes
async fn write(input_elem: WebElement<'_>, input_text: String) -> WebDriverResult<()> {
    println!("writing {} to element {}\n", input_text, input_elem.id().await?.unwrap());

    input_elem.clear().await?;
    input_elem.send_keys(input_text).await?;

    Ok(())
}
