use thirtyfour::prelude::*;
use std::time::Duration;
use tokio::time::sleep;

pub async fn open_browser_and_wait(base_url: &str, email: &str) -> Result<(), WebDriverError> {

    let email_trimmed = email.trim_matches('"').trim_matches('\'');
    let encoded_email = urlencoding::encode(email_trimmed);
    let url = format!("{}?user={}", base_url.trim_end_matches('/'), encoded_email);
    let mut caps = DesiredCapabilities::chrome();
    let _ = caps.add_arg("--no-sandbox");
    let _ = caps.add_arg("--disable-dev-shm-usage");
    let _ = caps.add_arg("--window-size=1920,1080");    
    let driver = WebDriver::new("http://localhost:21000", caps).await?;
    driver.get(&url).await?;
    sleep(Duration::from_secs(5)).await;
    driver.quit().await?;
    Ok(())
}