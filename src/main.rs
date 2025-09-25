#![windows_subsystem = "windows"]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::Duration;
use dirs::{home_dir};
use fs_extra::dir::{move_dir, CopyOptions};
use tray_item::{IconSource, TrayItem};
use auto_launch::AutoLaunch;

enum Message {
    Quit,
    Toggel,
}

#[derive(Default)]
struct Settings {
    toggel: bool,
    exit: bool,
}

fn main() {
    setup_auto_launch();

    let (tx, rx) = mpsc::channel::<Message>();

    let mut settings: Settings = Default::default();

    settings.toggel = true;
    settings.exit = false;

    let create_tray = |enable: bool, tx: mpsc::Sender<Message>| -> TrayItem{

        let mut tray = TrayItem::new("Slate", IconSource::Resource("app_icon")).unwrap();

        tray.add_menu_item("Sorting", || {
            sorting().expect("TODO: panic message");
        }).expect("TODO: panic message");

        tray.add_menu_item("
Restore sorting", || {
            extract_files_back().expect("TODO: panic message");
        }).expect("TODO: panic message");

        if enable {
            let tx_toggle = tx.clone();
            tray.add_menu_item("Disable", move || {
                tx_toggle.send(Message::Toggel).unwrap();
            })
                .unwrap();
        } else {
            let tx_toggle = tx.clone();
            tray.add_menu_item("Enable", move || {
                tx_toggle.send(Message::Toggel).unwrap();
            })
                .unwrap();
        }

        tray.inner_mut().add_separator().unwrap();

        let tx_exit = tx.clone();
        tray.add_menu_item("Quit", move || {
            tx_exit.send(Message::Quit).unwrap();
        })
            .unwrap();

        tray
    };

    let mut tray = create_tray(settings.toggel, tx.clone());

    update(&mut settings);

    loop {
        match rx.recv() {
            Ok(Message::Quit) => {
                settings.exit = true;
                break;
            }
            Ok(Message::Toggel) => {
                settings.toggel = !settings.toggel;
                drop(tray);
                tray = create_tray(settings.toggel, tx.clone());
            }
            _ => {}
        }
    }

}

fn setup_auto_launch() {
    let app_name = "Slate";

    // Получаем путь к текущему исполняемому файлу
    let app_path = std::env::current_exe()
        .expect("Failed to get current executable path")
        .to_string_lossy()
        .to_string();

    let auto = AutoLaunch::new(app_name, &app_path, &[] as &[&str]);

    // Включаем автозапуск (можно добавить проверку, если нужно)
    if let Err(e) = auto.enable() {
        eprintln!("Failed to enable auto-launch: {}", e);
    }
}

fn sorting() -> std::io::Result<()> {
    let files = return_all_files(get_downloads_path().to_str().unwrap());

    let mut folder_directory = get_downloads_path();
    folder_directory.push("folders");

    if !folder_directory.exists() {
        fs::create_dir(&folder_directory)?;
    }

    let mut target_directory = get_downloads_path();
    target_directory.push("files");

    if !target_directory.exists() {
        fs::create_dir(&target_directory)?;
    }


    for file in files {

        if file.file_name().unwrap() == "files" {
            continue;
        }

        if file.file_name().unwrap() == "folders" {
            continue;
        }

        if !file.is_file() {
            let options = CopyOptions::new();
            match move_dir(file, &folder_directory, &options) {
                Ok(_) => continue,
                Err(e) => println!("{:?}", e),
            }
            continue;
        }

        let file_name = match file.file_name() {
            Some(name) => name,
            None => {
                eprintln!("Ошибка: не могу получить имя файла для {}", file.display());
                continue;
            }
        };

        let extension = match file.extension() {
            Some(ext) => ext,
            None => {
                let no_ext_dir = target_directory.join("no_extension");
                if !no_ext_dir.exists() {
                    fs::create_dir(&no_ext_dir)?;
                }

                let new_path = no_ext_dir.join(file_name);
                if let Err(e) = fs::rename(&file, &new_path) {
                    eprintln!("Не удалось переместить файл без расширения {}: {}", file.display(), e);
                } else {
                    println!("Перемещен файл без расширения: {} -> {}", file.display(), new_path.display());
                }
                continue;
            }
        };

        let ext_str = match extension.to_str() {
            Some(ext) => ext,
            None => {
                eprintln!("Некорректное расширение у файла: {}", file.display());
                continue;
            }
        };

        let name = get_folder_name(ext_str);
        let ext_dir = target_directory.join(name);
        if !ext_dir.exists() {
            if let Err(e) = fs::create_dir(&ext_dir) {
                eprintln!("Не удалось создать папку {}: {}", ext_dir.display(), e);
                continue;
            }
            println!("Создана папка для расширения: {}", ext_dir.display());
        }

        let new_path = ext_dir.join(file_name);

        if new_path.exists() {
            println!("Файл уже существует: {}, пропускаем", new_path.display());
            continue;
        }

        match fs::rename(&file, &new_path) {
            Ok(_) => {
                println!("Успешно перемещен: {} -> {}", file.display(), new_path.display());
            }
            Err(e) => {
                eprintln!("Не удалось переместить файл {}: {}", file.display(), e);
                continue;
            }
        }
    }

    Ok(())
}

fn update(settings: &mut Settings) {
    loop {
        if settings.exit {
            break;
        }
        if settings.toggel {
            sorting().unwrap();
        }
        std::thread::sleep(Duration::from_secs(10));
    }
}

fn extract_files_back() -> std::io::Result<()> {
    let mut files_dir = get_downloads_path();
    files_dir.push("files");

    if !files_dir.exists() {
        println!("Папка 'files' не найдена в загрузках!");
        return Ok(());
    }

    let downloads_path = get_downloads_path();

    extract_files_recursive(&files_dir, &downloads_path)?;

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
            if let Err(e) = extract_files_recursive(&path, target_dir) {
                eprintln!("Ошибка при обработке папки {}: {}", path.display(), e);
            }

            if is_dir_empty(&path)? {
                if let Err(e) = fs::remove_dir(&path) {
                    eprintln!("Не удалось удалить папку {}: {}", path.display(), e);
                }
            }
        } else if path.is_file() {
            let file_name = path.file_name().unwrap();
            let new_path = target_dir.join(file_name);

            if new_path.exists() {
                println!("Файл уже существует: {}, пропускаем", new_path.display());
                continue;
            }

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

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("Ошибка чтения директории {}: {}", path, e);
            return files;
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(entry) => entry,
            Err(e) => {
                eprintln!("Ошибка чтения entry: {}", e);
                continue;
            }
        };

        let path = entry.path();
        files.push(path);
    }

    files
}