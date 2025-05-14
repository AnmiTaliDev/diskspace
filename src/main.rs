use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use std::time::Instant;
use std::cmp::Ordering;

// Структура для хранения информации о директории
struct DirInfo {
    size: u64,
    file_count: usize,
    largest_file: Option<(PathBuf, u64)>,
    file_types: BTreeMap<String, u64>,
}

impl DirInfo {
    fn new() -> Self {
        DirInfo {
            size: 0,
            file_count: 0,
            largest_file: None,
            file_types: BTreeMap::new(),
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let start_path = if args.len() > 1 {
        PathBuf::from(&args[1])
    } else {
        env::current_dir()?
    };

    println!("🔍 Анализ использования дискового пространства для: {:?}", start_path);
    println!("⏳ Подождите, идет сканирование...");
    
    let start_time = Instant::now();
    let mut dir_infos: BTreeMap<String, DirInfo> = BTreeMap::new();
    
    let total_info = scan_directory(&start_path, &mut dir_infos)?;
    
    let elapsed = start_time.elapsed();
    println!("\n✅ Сканирование завершено за {:.2} секунд", elapsed.as_secs_f32());
    println!("📊 Общий размер: {} МБ ({} файлов)\n", format_size(total_info.size), total_info.file_count);
    
    // Сортировка по размеру (по убыванию)
    let mut size_vec: Vec<(String, DirInfo)> = dir_infos.into_iter().collect();
    size_vec.sort_by(|a, b| b.1.size.cmp(&a.1.size));
    
    println!("📁 ТОП ДИРЕКТОРИИ ПО РАЗМЕРУ:");
    println!("{:<15} {:<12} {:<}", "РАЗМЕР", "ФАЙЛОВ", "ПУТЬ");
    println!("{:-<60}", "");
    
    // Выводим топ-15 директорий по размеру
    for (i, (path, info)) in size_vec.iter().take(15).enumerate() {
        let icon = match i {
            0 => "🔴",
            1 => "🟠",
            2 => "🟡",
            _ => "🔹",
        };
        
        println!("{} {:<15} {:<12} {:<}", 
                icon,
                format_size(info.size), 
                info.file_count,
                path);
    }
    
    // Анализ самых больших файлов
    println!("\n📄 САМЫЕ БОЛЬШИЕ ФАЙЛЫ:");
    println!("{:<15} {:<}", "РАЗМЕР", "ПУТЬ");
    println!("{:-<60}", "");
    
    let mut largest_files: Vec<(PathBuf, u64)> = Vec::new();
    for (_, info) in size_vec.iter() {
        if let Some(file_info) = &info.largest_file {
            largest_files.push(file_info.clone());
        }
    }
    
    largest_files.sort_by(|a, b| b.1.cmp(&a.1));
    for (path, size) in largest_files.iter().take(5) {
        println!("{:<15} {:<}", format_size(*size), path.display());
    }
    
    // Анализ типов файлов
    let mut file_type_totals: BTreeMap<String, u64> = BTreeMap::new();
    for (_, info) in size_vec.iter() {
        for (ext, size) in &info.file_types {
            *file_type_totals.entry(ext.clone()).or_insert(0) += size;
        }
    }
    
    let mut file_types_vec: Vec<(String, u64)> = file_type_totals.into_iter().collect();
    file_types_vec.sort_by(|a, b| b.1.cmp(&a.1));
    
    println!("\n📊 ИСПОЛЬЗОВАНИЕ ПО ТИПАМ ФАЙЛОВ:");
    println!("{:<15} {:<}", "РАЗМЕР", "ТИП");
    println!("{:-<60}", "");
    
    for (ext, size) in file_types_vec.iter().take(8) {
        let ext_name = if ext.is_empty() { "[без расширения]" } else { ext };
        println!("{:<15} {:<}", format_size(*size), ext_name);
    }
    
    // Советы по оптимизации
    generate_optimization_tips(&size_vec, &largest_files);
    
    Ok(())
}

fn scan_directory(dir: &Path, dir_infos: &mut BTreeMap<String, DirInfo>) -> io::Result<DirInfo> {
    let mut current_info = DirInfo::new();
    
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                // Рекурсивно обходим поддиректории
                let subdir_info = scan_directory(&path, dir_infos)?;
                current_info.size += subdir_info.size;
                current_info.file_count += subdir_info.file_count;
                
                // Обновляем информацию о самом большом файле
                if let Some(largest) = &subdir_info.largest_file {
                    match &current_info.largest_file {
                        Some(current_largest) if largest.1 > current_largest.1 => {
                            current_info.largest_file = Some(largest.clone());
                        },
                        None => current_info.largest_file = Some(largest.clone()),
                        _ => {}
                    }
                }
                
                // Сохраняем информацию о поддиректории
                if let Some(path_str) = path.to_str() {
                    dir_infos.insert(path_str.to_string(), subdir_info);
                }
            } else if path.is_file() {
                // Получаем размер файла
                if let Ok(metadata) = fs::metadata(&path) {
                    let file_size = metadata.len();
                    current_info.size += file_size;
                    current_info.file_count += 1;
                    
                    // Обновляем информацию о самом большом файле
                    match &current_info.largest_file {
                        Some(largest) if file_size > largest.1 => {
                            current_info.largest_file = Some((path.clone(), file_size));
                        },
                        None => current_info.largest_file = Some((path.clone(), file_size)),
                        _ => {}
                    }
                    
                    // Обновляем статистику по типам файлов
                    let extension = path.extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase();
                    
                    *current_info.file_types.entry(extension).or_insert(0) += file_size;
                }
            }
        }
    }
    
    Ok(current_info)
}

fn format_size(size: u64) -> String {
    if size < 1024 {
        format!("{} Б", size)
    } else if size < 1024 * 1024 {
        format!("{:.1} КБ", size as f64 / 1024.0)
    } else if size < 1024 * 1024 * 1024 {
        format!("{:.1} МБ", size as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} ГБ", size as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}

fn generate_optimization_tips(dirs: &Vec<(String, DirInfo)>, largest_files: &Vec<(PathBuf, u64)>) {
    println!("\n💡 СОВЕТЫ ПО ОПТИМИЗАЦИИ:");
    println!("{:-<60}", "");
    
    // Если есть очень большие директории
    if !dirs.is_empty() && dirs[0].1.size > 1024 * 1024 * 1024 {
        println!("🔸 Директория '{}' занимает {}, что составляет значительную часть дискового пространства.", 
            dirs[0].0, format_size(dirs[0].1.size));
    }
    
    // Советы по типам файлов
    let mut has_large_logs = false;
    let mut has_large_media = false;
    let mut has_downloads = false;
    
    for (path, info) in dirs.iter().take(5) {
        if path.to_lowercase().contains("log") && info.size > 100 * 1024 * 1024 {
            has_large_logs = true;
        }
        
        if path.to_lowercase().contains("download") {
            has_downloads = true;
        }
        
        for (ext, size) in &info.file_types {
            if (ext == "mp4" || ext == "mov" || ext == "avi") && *size > 500 * 1024 * 1024 {
                has_large_media = true;
            }
        }
    }
    
    if has_large_logs {
        println!("🔸 Обнаружены большие лог-файлы. Регулярная очистка логов может освободить значительное пространство.");
    }
    
    if has_large_media {
        println!("🔸 Медиафайлы занимают много места. Рассмотрите возможность переноса видео на внешний носитель или в облачное хранилище.");
    }
    
    if has_downloads {
        println!("🔸 Директория загрузок содержит много файлов. Очистка временных и ненужных загрузок может освободить пространство.");
    }
    
    // Советы по крупным файлам
    if !largest_files.is_empty() {
        let (path, size) = &largest_files[0];
        if *size > 1024 * 1024 * 1024 {
            println!("🔸 Файл '{}' занимает {}. Удаление или архивация этого файла значительно освободит место.", 
                path.display(), format_size(*size));
        }
    }
    
    println!("🔸 Рассмотрите использование инструментов сжатия для регулярно используемых файлов.");
    println!("🔸 Для системных файлов используйте команды очистки, специфичные для вашей ОС.");
}