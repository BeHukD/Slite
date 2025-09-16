use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use dirs::{download_dir, home_dir};

fn main() {
    let mut paths: Vec<String> = Vec::new();
    paths.push("files".to_string());
    welcome_operation(&paths);
}

fn welcome_operation(paths: &Vec<String>) {
    println!("Welcome, pls select operation");
    println!("Operations: Sorting (1), Return all files (2), Extract files back to Downloads (3)");
    let mut operation = String::new();
    io::stdin().read_line(&mut operation).unwrap();

    println!("Operation: {}", operation);

    match operation.as_str().trim() {
        "1" => {
            sorting_all_files(&paths)
        }
        "2" => {
            for i in return_all_files(get_downloads_path().to_str().unwrap()) {
                println!("{}", return_file_name(&i));
            }
        }
        "3" => {
            extract_files_back();
        }
        _ => {
            println!("Operation not correctable");
            return;
        }
    }
}

fn return_file_name(path: &Path) -> &str {
    path.file_name().unwrap().to_str().unwrap()
}

fn sorting_all_files(paths: &Vec<String>) {
    let mut dir = get_downloads_path();
    check_directory_for_sorting(&mut dir, &paths);
    sorting().unwrap_or_else(|e| eprintln!("Ошибка сортировки: {}", e));
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
                fs::rename(&file, &new_path)?;
                println!("Перемещен файл без расширения: {} -> {}", file.display(), new_path.display());
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
            fs::create_dir(&ext_dir)?;
            println!("Создана папка для расширения: {}", ext_dir.display());
        }

        // Перемещаем файл в соответствующую папку
        let new_path = ext_dir.join(file_name);

        // Проверяем, не существует ли уже файл с таким именем
        if new_path.exists() {
            println!("Файл уже существует: {}, пропускаем", new_path.display());
            continue;
        }

        fs::rename(&file, &new_path)?;
        println!("Успешно перемещен: {} -> {}", file.display(), new_path.display());
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
            extract_files_recursive(&path, target_dir)?;

            // Удаляем пустую папку после извлечения файлов
            if is_dir_empty(&path)? {
                fs::remove_dir(&path)?;
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

            fs::rename(&path, &new_path)?;
            println!("Извлечен: {} -> {}", path.display(), new_path.display());
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

fn check_directory_for_sorting(dir: &mut PathBuf, paths: &Vec<String>) {
    for i in paths {
        let mut reserv_dir = dir.clone();
        reserv_dir.push(i);
        if !reserv_dir.exists() {
            fs::create_dir(&reserv_dir).unwrap();
        }
    }
}

fn get_downloads_path() -> PathBuf {
    let mut path = home_dir().unwrap_or_else(|| PathBuf::from("."));

    if cfg!(windows) {
        path.push("Downloads");
    } else if cfg!(macos) {
        path.push("Downloads");
    } else {
        path.push("Загрузки"); // Для Linux
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