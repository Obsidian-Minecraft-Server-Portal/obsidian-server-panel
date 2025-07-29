export type FileTypeMap = {
    description: string;
    extensions: string[];
}

export const extensionFileTypeMap: FileTypeMap[] = [
    {
        extensions: ["doc", "docm", "docx"],
        description: "Word Document"
    },
    {
        extensions: ["dot", "dotx", "dotm"],
        description: "Word Template"
    },
    {
        extensions: ["xls", "xlsx", "xlsm", "xlsb"],
        description: "Excel Spreadsheet"
    },
    {
        extensions: ["xlt", "xltx", "xltm", "xlw"],
        description: "Excel Template"
    },
    {
        extensions: ["ppt", "pptx", "pptm"],
        description: "PowerPoint Presentation"
    },
    {
        extensions: ["pot", "potx", "potm"],
        description: "PowerPoint Template"
    },
    {
        extensions: ["odt"],
        description: "OpenDocument Text"
    },
    {
        extensions: ["ods"],
        description: "OpenDocument Spreadsheet"
    },
    {
        extensions: ["odp"],
        description: "OpenDocument Presentation"
    },
    {
        extensions: ["pdf"],
        description: "PDF Document"
    },
    {
        extensions: ["fdf", "xfdf", "pdx", "xdp"],
        description: "PDF Data File"
    },
    {
        extensions: ["jpg", "jpeg"],
        description: "JPEG Image"
    },
    {
        extensions: ["png"],
        description: "PNG Image"
    },
    {
        extensions: ["gif"],
        description: "GIF Image"
    },
    {
        extensions: ["bmp"],
        description: "Bitmap Image"
    },
    {
        extensions: ["svg"],
        description: "SVG Vector Image"
    },
    {
        extensions: ["webp"],
        description: "WebP Image"
    },
    {
        extensions: ["tiff", "tif"],
        description: "TIFF Image"
    },
    {
        extensions: ["ico"],
        description: "Icon File"
    },
    {
        extensions: ["psd", "psb", "pdd"],
        description: "Photoshop Document"
    },
    {
        extensions: ["ai", "ait", "art", "aip"],
        description: "Illustrator Document"
    },
    {
        extensions: ["indd", "indl", "indt", "indb"],
        description: "InDesign Document"
    },
    {
        extensions: ["mp4", "mpeg4"],
        description: "MP4 Video"
    },
    {
        extensions: ["webm"],
        description: "WebM Video"
    },
    {
        extensions: ["avi"],
        description: "AVI Video"
    },
    {
        extensions: ["mov", "qt"],
        description: "QuickTime Video"
    },
    {
        extensions: ["mkv"],
        description: "Matroska Video"
    },
    {
        extensions: ["flv"],
        description: "Flash Video"
    },
    {
        extensions: ["wmv"],
        description: "Windows Media Video"
    },
    {
        extensions: ["mpeg", "mpg"],
        description: "MPEG Video"
    },
    {
        extensions: ["m4v"],
        description: "M4V Video"
    },
    {
        extensions: ["3gp"],
        description: "3GP Video"
    },
    {
        extensions: ["ogv"],
        description: "OGG Video"
    },
    {
        extensions: ["mp3"],
        description: "MP3 Audio"
    },
    {
        extensions: ["wav"],
        description: "WAV Audio"
    },
    {
        extensions: ["ogg"],
        description: "OGG Audio"
    },
    {
        extensions: ["flac"],
        description: "FLAC Audio"
    },
    {
        extensions: ["aac"],
        description: "AAC Audio"
    },
    {
        extensions: ["m4a"],
        description: "M4A Audio"
    },
    {
        extensions: ["wma"],
        description: "Windows Media Audio"
    },
    {
        extensions: ["aiff"],
        description: "AIFF Audio"
    },
    {
        extensions: ["opus"],
        description: "Opus Audio"
    },
    {
        extensions: ["mid", "midi"],
        description: "MIDI Audio"
    },
    {
        extensions: ["html", "htm", "xhtml"],
        description: "HTML Document"
    },
    {
        extensions: ["css"],
        description: "CSS Stylesheet"
    },
    {
        extensions: ["scss", "sass"],
        description: "Sass Stylesheet"
    },
    {
        extensions: ["less"],
        description: "Less Stylesheet"
    },
    {
        extensions: ["js"],
        description: "JavaScript File"
    },
    {
        extensions: ["jsx"],
        description: "React JSX File"
    },
    {
        extensions: ["ts"],
        description: "TypeScript File"
    },
    {
        extensions: ["tsx"],
        description: "React TSX File"
    },
    {
        extensions: ["json"],
        description: "JSON File"
    },
    {
        extensions: ["jsonc", "json5"],
        description: "JSON with Comments"
    },
    {
        extensions: ["php"],
        description: "PHP File"
    },
    {
        extensions: ["asp", "aspx"],
        description: "ASP.NET File"
    },
    {
        extensions: ["jsp"],
        description: "JSP File"
    },
    {
        extensions: ["py"],
        description: "Python File"
    },
    {
        extensions: ["java"],
        description: "Java File"
    },
    {
        extensions: ["class"],
        description: "Java Class File"
    },
    {
        extensions: ["jar"],
        description: "Java Archive"
    },
    {
        extensions: ["c"],
        description: "C File"
    },
    {
        extensions: ["cpp", "cc", "cxx"],
        description: "C++ File"
    },
    {
        extensions: ["h", "hpp"],
        description: "C/C++ Header"
    },
    {
        extensions: ["cs"],
        description: "C# File"
    },
    {
        extensions: ["go"],
        description: "Go File"
    },
    {
        extensions: ["rs"],
        description: "Rust File"
    },
    {
        extensions: ["rb"],
        description: "Ruby File"
    },
    {
        extensions: ["swift"],
        description: "Swift File"
    },
    {
        extensions: ["kt", "kts"],
        description: "Kotlin File"
    },
    {
        extensions: ["zip"],
        description: "ZIP Archive"
    },
    {
        extensions: ["rar"],
        description: "RAR Archive"
    },
    {
        extensions: ["7z"],
        description: "7-Zip Archive"
    },
    {
        extensions: ["tar"],
        description: "TAR Archive"
    },
    {
        extensions: ["gz", "gzip"],
        description: "GZip Archive"
    },
    {
        extensions: ["bz2", "bzip2"],
        description: "BZip2 Archive"
    },
    {
        extensions: ["xz"],
        description: "XZ Archive"
    },
    {
        extensions: ["tgz"],
        description: "Compressed TAR"
    },
    {
        extensions: ["zst"],
        description: "Zstandard Archive"
    },
    {
        extensions: ["txt"],
        description: "Text File"
    },
    {
        extensions: ["md", "markdown"],
        description: "Markdown Document"
    },
    {
        extensions: ["rtf"],
        description: "Rich Text Format"
    },
    {
        extensions: ["csv"],
        description: "CSV Spreadsheet"
    },
    {
        extensions: ["xml"],
        description: "XML Document"
    },
    {
        extensions: ["yaml", "yml"],
        description: "YAML File"
    },
    {
        extensions: ["toml"],
        description: "TOML File"
    },
    {
        extensions: ["ini", "cfg", "conf"],
        description: "Configuration File"
    },
    {
        extensions: ["log"],
        description: "Log File"
    },
    {
        extensions: ["exe", "com"],
        description: "Windows Executable"
    },
    {
        extensions: ["msi"],
        description: "Windows Installer"
    },
    {
        extensions: ["app"],
        description: "macOS Application"
    },
    {
        extensions: ["dmg"],
        description: "macOS Disk Image"
    },
    {
        extensions: ["pkg"],
        description: "macOS Package"
    },
    {
        extensions: ["deb"],
        description: "Debian Package"
    },
    {
        extensions: ["rpm"],
        description: "RPM Package"
    },
    {
        extensions: ["appimage"],
        description: "AppImage"
    },
    {
        extensions: ["apk"],
        description: "Android Package"
    },
    {
        extensions: ["sh", "bash"],
        description: "Shell Script"
    },
    {
        extensions: ["bat", "cmd"],
        description: "Batch File"
    },
    {
        extensions: ["ps1"],
        description: "PowerShell Script"
    },
    {
        extensions: ["sql"],
        description: "SQL Script"
    },
    {
        extensions: ["sqlite", "db"],
        description: "SQLite Database"
    },
    {
        extensions: ["mdb", "accdb"],
        description: "Access Database"
    },
    {
        extensions: ["ttf"],
        description: "TrueType Font"
    },
    {
        extensions: ["otf"],
        description: "OpenType Font"
    },
    {
        extensions: ["woff"],
        description: "Web Font"
    },
    {
        extensions: ["woff2"],
        description: "Web Font 2.0"
    },
    {
        extensions: ["eot"],
        description: "Embedded Font"
    },
    {
        extensions: ["obj"],
        description: "3D Object"
    },
    {
        extensions: ["fbx"],
        description: "FBX 3D Model"
    },
    {
        extensions: ["glb", "gltf"],
        description: "glTF 3D Model"
    },
    {
        extensions: ["dll"],
        description: "Windows Library"
    },
    {
        extensions: ["so"],
        description: "Shared Object"
    },
    {
        extensions: ["o"],
        description: "Object File"
    },
    {
        extensions: ["lib"],
        description: "C/C++ Library File"
    },
    {
        extensions: ["dylib"],
        description: "macOS Library"
    },
    {
        extensions: ["iso"],
        description: "Disk Image"
    },
    {
        extensions: ["dat", "bin"],
        description: "Binary Data"
    },
    {
        extensions: ["properties", "prop"],
        description: "Properties File"
    }
];

export function getFileType(path: string): FileTypeMap | undefined
{
    let extension = getFileExtension(path);
    if (extension)
        return extensionFileTypeMap.find(e => e.extensions.includes(extension)) ?? undefined;
    return undefined;
}

export function getFileExtension(path: string): string | undefined
{
    console.debug(`Processing file path: ${path}`);
    const filename = path.replace(/\\/g, "/").split("/").pop()?.toLowerCase().trim();
    if (!filename)
    {
        console.error("No filename found");
        return undefined;
    }
    let multi_extension = filename.split(".").slice(1);
    let extension = multi_extension.length > 0 ? multi_extension.join(".") : "";
    console.debug(`Checking multi-extension: ${extension}`);
    let multi_extension_match = extensionFileTypeMap.find(e => e.extensions.includes(extension));
    if (multi_extension_match)
    {
        console.debug(`Found multi-extension match: ${extension}`);
        return extension;
    } else
    {
        let single_extension = filename.split(".").pop()?.toLowerCase().trim();
        if (!single_extension)
        {
            console.warn("No single extension found");
            return undefined;
        }
        console.debug(`Checking single extension: ${single_extension}`);
        let single_extension_match = extensionFileTypeMap.find(e => e.extensions.includes(single_extension));
        if (single_extension_match)
        {
            console.debug(`Found single extension match: ${single_extension}`);
            return single_extension;
        }
        console.warn(`No extension match found for filename: ${filename}`);
    }
    return multi_extension.join(".");
}

export function isTextFile(path: string): boolean
{
    const fileType = getFileType(path);
    if (!fileType) return false;
    return fileType.extensions.some(ext => ["txt", "md", "json", "xml", "csv", "yaml", "yml", "toml", "properties", "ini", "cfg", "conf", "log", "sh", "bash", "bat", "cmd", "ps1", "sql", "html", "htm", "xhtml", "css", "scss", "sass", "less", "js", "jsx", "ts", "tsx", "php", "py", "java", "c", "cpp", "h", "hpp", "cs", "go", "rs", "rb", "swift", "kt", "kts"].includes(ext));
}

export function getMonacoLanguage(path: string): string | undefined
{
    const fileType = getFileType(path);
    if (!fileType) return undefined;
    switch (fileType.description)
    {
        case "JavaScript File":
            return "javascript";
        case "TypeScript File":
            return "typescript";
        case "JSON File":
            return "json";
        case "HTML Document":
            return "html";
        case "CSS Stylesheet":
            return "css";
        case "Python File":
            return "python";
        case "Java File":
            return "java";
        case "C File":
            return "c";
        case "C++ File":
            return "cpp";
        case "C# File":
            return "csharp";
        case "Go File":
            return "go";
        case "Rust File":
            return "rust";
        case "Ruby File":
            return "ruby";
        case "Swift File":
            return "swift";
        case "Kotlin File":
            return "kotlin";
        case "Properties File":
            return "properties";
        default:
            console.warn(`No Monaco language mapping for file type: ${fileType.description}`);
            return undefined;
    }
}