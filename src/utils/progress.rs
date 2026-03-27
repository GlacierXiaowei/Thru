use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

pub fn create_upload_bar(total_size: u64, file_name: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    
    pb.set_style(
        ProgressStyle::with_template(
            &format!("📤 Sending {}\n[{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{percent}}%  {{bytes}}/{{total_bytes}}  {{bytes_per_sec}}  ETA: {{eta}}", 
                file_name)
        )
        .unwrap()
        .progress_chars("#>-")
    );
    
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}

pub fn create_download_bar(total_size: u64, file_name: &str) -> ProgressBar {
    let pb = ProgressBar::new(total_size);
    
    pb.set_style(
        ProgressStyle::with_template(
            &format!("📥 Receiving {}\n[{{elapsed_precise}}] [{{bar:40.green/blue}}] {{percent}}%  {{bytes}}/{{total_bytes}}  {{bytes_per_sec}}  ETA: {{eta}}", 
                file_name)
        )
        .unwrap()
        .progress_chars("#>-")
    );
    
    pb.enable_steady_tick(Duration::from_millis(100));
    pb
}