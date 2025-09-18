import {Icon} from "@iconify-icon/react";
import {FilesystemEntry} from "../../../../ts/filesystem.ts";
import {getFileExtension} from "../../../../ts/file-type-match.ts";

export   function FileEntryIcon({entry}: { entry: FilesystemEntry })
{
    const extension = getFileExtension(entry.filename);
    switch (extension)
    {
        /// Excel File Extensions
        case "csv":
        case "xls":
        case "xlsx":
        case "xlsb":
        case "xlsm":
        case "xlt":
        case "xltx":
        case "xltm":
        case "xlw":
        case "ods":
            return <Icon
                icon={"file-icons:microsoft-excel"}
                className={"text-2xl text-green-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Word Document Extensions 
        case "docx":
        case "doc":
        case "docm":
        case "dot":
        case "dotx":
        case "dotm":
        case "rtf":
            return <Icon
                icon={"vscode-icons:file-type-word"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Adobe Creative Suite Extensions
        case "psd":
        case "psb":
        case "pdd":
            return <Icon
                icon={"logos:adobe-photoshop"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        case "ai":
        case "ait":
        case "art":
        case "aip":
            return <Icon
                icon={"logos:adobe-illustrator"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        case "indd":
        case "indl":
        case "indt":
        case "indb":
            return <Icon
                icon={"logos:adobe-indesign"}
                className={"text-2xl text-pink-600"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        case "aep":
        case "aet":
        case "aepx":
            return <Icon
                icon={"logos:adobe-after-effects"}
                className={"text-2xl text-purple-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        case "prproj":
        case "prel":
        case "psq":
            return <Icon
                icon={"logos:adobe-premiere"}
                className={"text-2xl text-violet-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Video File Extensions
        case "mp4":
        case "mkv":
        case "mpeg":
        case "mpeg4":
        case "webm":
        case "avi":
        case "mov":
        case "flv":
        case "wmv":
        case "m4v":
        case "3gp":
        case "vob":
        case "ogv":
        case "qt":
        case "rm":
        case "rmvb":
        case "asf":
        case "m2ts":
            return <Icon
                icon={"lets-icons:video-file-fill"}
                className={"text-2xl text-fuchsia-400"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Audio File Extensions
        case "mp3":
        case "wav":
        case "ogg":
        case "flac":
        case "aac":
        case "m4a":
        case "wma":
        case "aiff":
        case "alac":
        case "opus":
        case "mid":
        case "midi":
            return <Icon
                icon={"fa6-solid:file-audio"}
                className={"text-2xl text-emerald-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Image File Extensions
        case "png":
        case "jpg":
        case "jpeg":
        case "bmp":
        case "webp":
        case "gif":
        case "tiff":
        case "svg":
        case "ico":
        case "ppm":
        case "pgm":
        case "pbm":
        case "hdr":
        case "exr":
        case "dds":
        case "tga":
            return <Icon
                icon={"fa6-solid:file-image"}
                className={"text-2xl text-blue-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// PDF Document Extensions
        case "pdf":
        case "fdf":
        case "xfdf":
        case "pdx":
        case "xdp":
            return <Icon
                icon={"fa6-solid:file-pdf"}
                className={"text-2xl text-cyan-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Binary File Extensions
        case "dat":
        case "nbt":
        case "data":
        case "bin":
        case "hex":
        case "iso":
        case "img":
        case "raw":
        case "dll":
        case "so":
        case "dylib":
        case "sys":
            return <Icon
                icon={"bi:file-earmark-binary-fill"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Executable File Extensions
        case "exe":
        case "sh":
        case "bat":
        case "ps1":
        case "command":
        case "cmd":
        case "com":
        case "run":
        case "app":
        case "msi":
        case "deb":
        case "rpm":
        case "apk":
        case "dmg":
        case "pkg":
        case "appimage":
            return <Icon
                icon={"bi:file-earmark-binary-fill"}
                className={"text-2xl text-red-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Java Related Extensions
        case "java":
        case "class":
            return <Icon
                icon={"fluent:document-java-16-filled"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;
        case "jar":
        case "jmod":
            return <Icon
                icon={"skill-icons:java-light"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// JavaScript/TypeScript Extensions
        case "js":
        case "jsx":
            return <Icon
                icon={"skill-icons:javascript"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;
        case "ts":
        case "tsx":
            return <Icon
                icon={"logos:typescript-icon"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Rust Extensions
        case "rs":
            return <Icon
                icon={"fa6-brands:rust"}
                className={"text-2xl text-orange-600"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Text & Configuration File Extensions
        case "txt":
        case "log":
        case "ini":
        case "cfg":
        case "conf":
        case "properties":
        case "md":
        case "markdown":
        case "yml":
        case "yaml":
        case "toml":
            return <Icon
                icon={"tabler:file-text-filled"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// JSON File Extensions
        case "json5":
        case "jsonc":
        case "json":
            return <Icon
                icon={"bxs:file-json"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        /// Archive File Extensions
        case "zip":
        case "tar":
        case "gz":
        case "7z":
        case "rar":
        case "bz2":
        case "xz":
        case "tgz":
        case "war":
        case "bzip2":
        case "gzip":
        case "lz":
        case "lzma":
        case "lzo":
        case "zst":
            return <Icon
                icon={"solar:zip-file-bold"}
                className={"text-2xl text-yellow-300"}
                data-directory={"false"}
                aria-hidden="true"
            />;
        // HTML/Web Files
        case "html":
        case "htm":
        case "xhtml":
        case "cshtml":
        case "razor":
        case "php":
        case "aspx":
        case "jsp":
            return <Icon
                icon={"vscode-icons:file-type-html"}
                className={"text-2xl text-orange-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // CSS and Styling
        case "css":
        case "scss":
        case "sass":
        case "less":
        case "styl":
        case "postcss":
            return <Icon
                icon={"vscode-icons:file-type-css"}
                className={"text-2xl text-blue-400"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Web Config Files
        case "htaccess":
        case "browserslistrc":
        case "eslintrc":
        case "eslintignore":
        case "stylelintrc":
        case "babelrc":
            return <Icon
                icon={"mdi:file-settings-cog"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Web Build/Package Files
        case "npmrc":
        case "yarnrc":
            return <Icon
                icon={"logos:npm-icon"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Package JSON files
        case "package.json":
        case "package-lock.json":
        case "yarn.lock":
            return <Icon
                icon={"vscode-icons:file-type-npm"}
                className={"text-2xl text-red-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // TypeScript Config
        case "tsconfig.json":
        case "jsconfig.json":
            return <Icon
                icon={"vscode-icons:file-type-typescript-official"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // TypeScript Declaration Files
        case "d.ts":
            return <Icon
                icon={"logos:typescript-icon"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Build Config Files
        case "webpack.config.js":
        case "vite.config.js":
        case "rollup.config.js":
            return <Icon
                icon={"logos:webpack"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Font Files
        case "woff":
        case "woff2":
        case "ttf":
        case "otf":
        case "eot":
            return <Icon
                icon={"fa6-solid:font"}
                className={"text-2xl text-purple-400"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Web Manifest Files
        case "webmanifest":
        case "manifest.json":
            return <Icon
                icon={"mdi:web"}
                className={"text-2xl text-blue-600"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Service Worker
        case "sw.js":
            return <Icon
                icon={"mdi:worker"}
                className={"text-2xl text-gray-600"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // GraphQL
        case "graphql":
        case "gql":
            return <Icon
                icon={"logos:graphql"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Certificate/Security Files
        case "cer":
        case "crt":
        case "key":
        case "csr":
        case "pem":
            return <Icon
                icon={"teenyicons:certificate-solid"}
                className={"text-2xl "}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Database Files
        case "sql":
        case "sqlite":
        case "db":
            return <Icon
                icon={"mdi:database"}
                className={"text-2xl text-blue-700"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Microsoft Access Files
        case "mdb":
        case "accdb":
            return <Icon
                icon={"vscode-icons:file-type-access"}
                className={"text-2xl text-red-700"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // MongoDB Files
        case "mongodb":
            return <Icon
                icon={"skill-icons:mongodb"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // 3D/WebGL Files
        case "glb":
        case "gltf":
        case "obj":
        case "fbx":
        case "mtl":
            return <Icon
                icon={"mdi:cube-outline"}
                className={"text-2xl text-purple-600"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Web-specific Document Formats
        case "xml":
        case "xsd":
        case "dtd":
            return <Icon
                icon={"vscode-icons:file-type-xml"}
                className={"text-2xl text-orange-600"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Feed Formats
        case "rss":
        case "atom":
            return <Icon
                icon={"mdi:rss"}
                className={"text-2xl text-orange-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        // Calendar Format
        case "ics":
            return <Icon
                icon={"mdi:calendar"}
                className={"text-2xl text-blue-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        case "pug":
        case "jade":
            return <Icon
                icon={"logos:pug"}
                className={"text-2xl"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        case "ejs":
        case "njk":
            return <Icon
                icon={"mdi:code-tags"}
                className={"text-2xl text-yellow-600"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        case "twig":
        case "blade.php":
            return <Icon
                icon={"mdi:language-php"}
                className={"text-2xl text-purple-500"}
                data-directory={"false"}
                aria-hidden="true"
            />;

        default:
            return <Icon
                icon={entry.is_dir ? "mage:folder-fill" : "mage:file-fill"}
                className={"text-2xl data-[directory=true]:text-blue-500"}
                data-directory={entry.is_dir ? "true" : "false"}
                aria-hidden="true"
            />;
    }
}