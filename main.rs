/*
[dependencies]
tera = "1.20.0"
serde = { version = "1.0", features = ["derive"] }
headless_chrome = { version = "1.0.13", default-features = false, features = ["fetch"] }
*/

use serde::Serialize;
use tera::{Context, Tera};
use std::fs::File;
use std::io::Write;
use headless_chrome::{Browser, LaunchOptions};

// Define the data structures for our student report
#[derive(Serialize)]
struct Lesson {
    name: String,
    grade: u8,
}

#[derive(Serialize)]
struct StudentReport {
    student_name: String,
    student_id: String,
    term: String,
    lessons: Vec<Lesson>,
    average: f32,
    passed: bool,
}

// HTML template with embedded CSS styling for professional look
const HTML_TEMPLATE: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <style>
        body { font-family: 'Helvetica Neue', Arial, sans-serif; margin: 40px; color: #333; }
        .header { text-align: center; border-bottom: 3px solid #4A90E2; padding-bottom: 20px; }
        .school-title { font-size: 24px; font-weight: bold; color: #2C3E50; margin: 0; }
        .report-title { font-size: 18px; color: #7F8C8D; margin: 5px 0 0 0; }
        .info-container { margin: 30px 0; display: flex; justify-content: space-between; background: #F8F9F9; padding: 15px; border-radius: 5px; }
        .info-block { font-size: 14px; }
        table { width: 100%; border-collapse: collapse; margin-top: 20px; }
        th, td { border: 1px solid #BDC3C7; padding: 12px; text-align: left; }
        th { background-color: #34495E; color: white; }
        tr:nth-child(even) { background-color: #F2F4F4; }
        .total-row { font-weight: bold; background-color: #EAEDED !important; }
        .status-container { margin-top: 30px; font-size: 16px; font-weight: bold; }
        .status-passed { color: #27AE60; }
        .status-failed { color: #C0392B; }
    </style>
</head>
<body>
    <div class="header">
        <div class="school-title">RUST AKADEMİSİ</div>
        <div class="report-title">Öğrenci Dönem Sonu Başarı Raporu</div>
    </div>
    
    <div class="info-container">
        <div class="info-block">
            <strong>Öğrenci Adı:</strong> {{ student_name }}<br>
            <strong>Öğrenci No:</strong> {{ student_id }}
        </div>
        <div class="info-block" style="text-align: right;">
            <strong>Dönem:</strong> {{ term }}<br>
            <strong>Tarih:</strong> 2024-06-15
        </div>
    </div>

    <table>
        <thead>
            <tr>
                <th>Ders Adı</th>
                <th style="text-align: right;">Notu</th>
            </tr>
        </thead>
        <tbody>
            {% for lesson in lessons %}
            <tr>
                <td>{{ lesson.name }}</td>
                <td style="text-align: right;">{{ lesson.grade }}</td>
            </tr>
            {% endfor %}
            <tr class="total-row">
                <td>Genel Ortalama</td>
                <td style="text-align: right;">{{ average | round(precision=2) }}</td>
            </tr>
        </tbody>
    </table>

    <div class="status-container">
        Durum: 
        {% if passed %}
            <span class="status-passed">GEÇTİ (Başarılı)</span>
        {% else %}
            <span class="status-failed">KALDI (Başarısız)</span>
        {% endif %}
    </div>
</body>
</html>
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println="[1/4] Dinamik öğrenci verileri hazırlanıyor...";
    
    let lessons = vec![
        Lesson { name: "Rust Programlama Giriş".to_string(), grade: 95 },
        Lesson { name: "Sistem Programlama ve Bellek Yönetimi".to_string(), grade: 88 },
        Lesson { name: "Asenkron Programlama (Tokio)".to_string(), grade: 78 },
    ];
    
    let average: f32 = lessons.iter().map(|l| l.grade as f32).sum::<f32>() / lessons.len() as f32;
    
    let report = StudentReport {
        student_name: "Can Demir".to_string(),
        student_id: "RUST-2024-0982".to_string(),
        term: "2024 Bahar Dönemi".to_string(),
        lessons,
        average,
        passed: average >= 60.0,
    };

    println!("[2/4] HTML şablonu Tera ile derleniyor...");
    let mut tera = Tera::default();
    tera.add_raw_template("report_template", HTML_TEMPLATE)?;
    
    let context = Context::from_serialize(&report)?;
    let rendered_html = tera.render("report_template", &context)?;

    // Save rendered HTML to a temporary file
    let temp_html_path = "temp_report.html";
    let mut file = File::create(temp_html_path)?;
    file.write_all(rendered_html.as_bytes())?;

    println!("[3/4] Headless Chrome başlatılıyor ve PDF üretiliyor...");
    // Launch browser (will auto-download portable Chromium if not found due to 'fetch' feature)
    let browser = Browser::new(
        LaunchOptions::default()
    )?;
    
    let tab = browser.new_tab()?;
    
    // Navigate to the local temporary HTML file
    let absolute_path = std::fs::canonicalize(temp_html_path)?;
    let file_url = format!("file://{}", absolute_path.to_str().unwrap());
    
    tab.navigate_to(&file_url)?;
    tab.wait_until_navigated()?;

    // Export the rendered page to PDF
    let pdf_bytes = tab.print_to_pdf(None)?;
    
    let output_pdf_path = "ogrenci_raporu.pdf";
    let mut pdf_file = File::create(output_pdf_path)?;
    pdf_file.write_all(&pdf_bytes)?;

    println!("[4/4] Geçici dosyalar temizleniyor...");
    std::fs::remove_file(temp_html_path)?;

    println!("\nBaşarılı! Öğrenci raporu oluşturuldu: {}", output_pdf_path);
    Ok(())
}