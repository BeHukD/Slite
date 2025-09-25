![Logo](resource/logo.png)

# Slate - Auto File Organizer

My first Rust project! A simple tool that automatically organizes files in your Downloads folder by category (Images, Documents, Archives, etc.).

## Features

- **Automatic sorting:** Sorts files by type (images, documents, videos, archives, code, etc.)
- **Tray interface:** Runs in system tray with enable/disable toggle
- **Manual control:** Sort now or restore files to their original location
- **Safe:** Only moves files, doesn't delete them

## Installation for users

1. **Download Slate.exe** from https://github.com/BeHukD/Slite/releases
2. **Run Slate.exe**
3. **Open Tray**

## Installation for development

1. **Install Rust** from [rust-lang.org](https://rust-lang.org)
2. **Clone & build:**
   ```bash
   git clone https://github.com/yourusername/slate.git
   cd slate
   cargo build --release