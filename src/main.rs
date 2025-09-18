#![windows_subsystem = "windows"]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use dirs::{home_dir};
use tray_item::{IconSource, TrayItem};

enum Message {
    Quit,
}

fn main() {
    let mut tray = TrayItem::new("Example", IconSource::Resource("app_icon")).unwrap();

    tray.add_menu_item("Sorting", || {
        sorting().expect("TODO: panic message");
    }).expect("TODO: panic message");

    tray.add_menu_item("UnSorting", || {
        extract_files_back().expect("TODO: panic message");
    }).expect("TODO: panic message");

    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = mpsc::sync_channel(1);

    let quit_tx = tx.clone();
    tray.add_menu_item("Quit", move || {
        quit_tx.send(Message::Quit).unwrap();
    })
        .unwrap();

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                println!("Quit");
                break;
            }
            _ => {}
        }
    }

    //let mut paths: Vec<String> = Vec::new();
    //paths.push("files".to_string());
    //welcome_operation(&paths);
}

fn sorting() -> std::io::Result<()> {
    let files = return_all_files(get_downloads_path().to_str().unwrap());
    let mut target_directory = get_downloads_path();
    target_directory.push("files");

    // Создаем основную папку "files" если её нет
    if !target_directory.exists() {
        fs::create_dir(&target_directory)?;
        println!("Создана основная папка: {}", target_directory.display());
    }

    for file in files {
        // Пропускаем саму папку "files"
        if file.file_name().unwrap() == "files" {
            continue;
        }

        // Получаем только имя файла (без пути)
        let file_name = match file.file_name() {
            Some(name) => name,
            None => {
                eprintln!("Ошибка: не могу получить имя файла для {}", file.display());
                continue;
            }
        };

        // Получаем расширение файла
        let extension = match file.extension() {
            Some(ext) => ext,
            None => {
                // Файлы без расширения перемещаем в special папку
                let no_ext_dir = target_directory.join("no_extension");
                if !no_ext_dir.exists() {
                    fs::create_dir(&no_ext_dir)?;
                }

                let new_path = no_ext_dir.join(file_name);
                // Обрабатываем ошибку перемещения
                if let Err(e) = fs::rename(&file, &new_path) {
                    eprintln!("Не удалось переместить файл без расширения {}: {}", file.display(), e);
                } else {
                    println!("Перемещен файл без расширения: {} -> {}", file.display(), new_path.display());
                }
                continue;
            }
        };

        // Получаем расширение как строку
        let ext_str = match extension.to_str() {
            Some(ext) => ext,
            None => {
                eprintln!("Некорректное расширение у файла: {}", file.display());
                continue;
            }
        };

        // Создаем папку для расширения
        let name = get_folder_name(ext_str);
        let ext_dir = target_directory.join(name);
        if !ext_dir.exists() {
            if let Err(e) = fs::create_dir(&ext_dir) {
                eprintln!("Не удалось создать папку {}: {}", ext_dir.display(), e);
                continue;
            }
            println!("Создана папка для расширения: {}", ext_dir.display());
        }

        // Перемещаем файл в соответствующую папку
        let new_path = ext_dir.join(file_name);

        // Проверяем, не существует ли уже файл с таким именем
        if new_path.exists() {
            println!("Файл уже существует: {}, пропускаем", new_path.display());
            continue;
        }

        // Пытаемся переместить файл, обрабатываем ошибки
        match fs::rename(&file, &new_path) {
            Ok(_) => {
                println!("Успешно перемещен: {} -> {}", file.display(), new_path.display());
            }
            Err(e) => {
                eprintln!("Не удалось переместить файл {}: {}", file.display(), e);
                // Продолжаем обработку следующих файлов
                continue;
            }
        }
    }

    Ok(())
}

fn extract_files_back() -> std::io::Result<()> {
    let mut files_dir = get_downloads_path();
    files_dir.push("files");

    if !files_dir.exists() {
        println!("Папка 'files' не найдена в загрузках!");
        return Ok(());
    }

    let downloads_path = get_downloads_path();

    // Рекурсивно обходим все файлы в папке files
    extract_files_recursive(&files_dir, &downloads_path)?;

    // Удаляем пустую папку files
    if is_dir_empty(&files_dir)? {
        fs::remove_dir(&files_dir)?;
        println!("Папка 'files' удалена");
    }

    println!("Все файлы успешно извлечены обратно в загрузки!");
    Ok(())
}

fn extract_files_recursive(source_dir: &Path, target_dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Рекурсивно обрабатываем подпапки
            if let Err(e) = extract_files_recursive(&path, target_dir) {
                eprintln!("Ошибка при обработке папки {}: {}", path.display(), e);
            }

            // Удаляем пустую папку после извлечения файлов
            if is_dir_empty(&path)? {
                if let Err(e) = fs::remove_dir(&path) {
                    eprintln!("Не удалось удалить папку {}: {}", path.display(), e);
                }
            }
        } else if path.is_file() {
            // Перемещаем файл в целевую директорию
            let file_name = path.file_name().unwrap();
            let new_path = target_dir.join(file_name);

            // Проверяем на конфликт имен
            if new_path.exists() {
                println!("Файл уже существует: {}, пропускаем", new_path.display());
                continue;
            }

            // Пытаемся переместить файл, обрабатываем ошибки
            match fs::rename(&path, &new_path) {
                Ok(_) => {
                    println!("Извлечен: {} -> {}", path.display(), new_path.display());
                }
                Err(e) => {
                    eprintln!("Не удалось извлечь файл {}: {}", path.display(), e);
                    // Продолжаем обработку следующих файлов
                    continue;
                }
            }
        }
    }

    Ok(())
}

fn is_dir_empty(path: &Path) -> std::io::Result<bool> {
    let mut entries = fs::read_dir(path)?;
    Ok(entries.next().is_none())
}

fn get_folder_name(format: &str) -> String {
    return match format.to_lowercase().as_str() {
        // Images
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "tiff" | "webp" | "svg" | "ico" | "raw" | "heic" => "images".to_string(),

        // Documents
        "txt" | "doc" | "docx" | "pdf" | "rtf" | "odt" | "pages" => "documents".to_string(),

        // Spreadsheets
        "xls" | "xlsx" | "csv" | "ods" | "numbers" => "spreadsheets".to_string(),

        // Presentations
        "ppt" | "pptx" | "key" | "odp" => "presentations".to_string(),

        // Archives
        "zip" | "rar" | "7z" | "tar" | "gz" | "bz2" => "archives".to_string(),

        // Audio
        "mp3" | "wav" | "flac" | "aac" | "ogg" | "wma" | "m4a" => "audio".to_string(),

        // Video
        "mp4" | "avi" | "mkv" | "mov" | "wmv" | "flv" | "webm" | "m4v" => "video".to_string(),

        // Code
        "py" => "python".to_string(),
        "rs" => "rust".to_string(),
        "js" | "jsx" => "javascript".to_string(),
        "ts" | "tsx" => "typescript".to_string(),
        "java" => "java".to_string(),
        "cpp" | "cc" | "cxx" => "cpp".to_string(),
        "c" => "c".to_string(),
        "html" | "htm" => "html".to_string(),
        "css" => "css".to_string(),
        "php" => "php".to_string(),
        "rb" => "ruby".to_string(),
        "go" => "go".to_string(),
        "swift" => "swift".to_string(),
        "kt" | "kts" => "kotlin".to_string(),
        "json" => "json".to_string(),
        "xml" => "xml".to_string(),
        "yml" | "yaml" => "yaml".to_string(),
        "toml" => "toml".to_string(),
        "sql" => "sql".to_string(),
        "sh" | "bash" => "shell".to_string(),
        "ps1" => "powershell".to_string(),
        "bat" | "cmd" => "batch".to_string(),

        // Executables
        "exe" | "msi" => "executables".to_string(),
        "dmg" => "mac_installers".to_string(),
        "deb" | "rpm" => "linux_packages".to_string(),

        // Fonts
        "ttf" | "otf" | "woff" | "woff2" => "fonts".to_string(),

        // Ebooks
        "epub" | "mobi" | "azw3" => "ebooks".to_string(),

        // Torrents
        "torrent" => "torrents".to_string(),

        // Config
        "ini" | "conf" | "cfg" => "config".to_string(),

        // Logs
        "log" => "logs".to_string(),

        // Database
        "db" | "sqlite" | "mdb" => "databases".to_string(),

        // Photoshop
        "psd" => "photoshop".to_string(),

        // Vector graphics
        "ai" | "eps" => "vector_graphics".to_string(),

        // 3D models
        "obj" | "stl" | "fbx" | "blend" => "3d_models".to_string(),

        // Default - use extension as folder name
        _ => format.to_lowercase()
    }
}

fn get_downloads_path() -> PathBuf {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("."));

    if cfg!(windows) {
        path.push("Downloads");
    }

    path
}

fn return_all_files(path: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();

    // Обрабатываем Result от read_dir
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Ошибка чтения директории {}: {}", path, e);
            return files; // Возвращаем пустой вектор
        }
    };

    for entry in entries {
        // Обрабатываем Result от каждого entry
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Ошибка чтения entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    files
}