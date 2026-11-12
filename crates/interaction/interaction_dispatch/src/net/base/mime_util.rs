// Copyright 2012 The Chromium Authors
// Use of this source code is governed by a BSD-style license.
// Chromium: 6c1d401fcca5e1b0030563a90c2f2bba168e0c15
// Source: net/base/mime_util.cc:169-287,316-320,357-400.
// The well-known lookup does not consult the platform MIME database.
const MAPPINGS: &[(&str, &str)] = &[
    ("video/webm", "webm"),
    ("audio/mpeg", "mp3"),
    ("application/wasm", "wasm"),
    ("application/x-chrome-extension", "crx"),
    ("application/xhtml+xml", "xhtml,xht,xhtm"),
    ("audio/flac", "flac"),
    ("audio/matroska", "mka"),
    ("audio/mp3", "mp3"),
    ("audio/ogg", "ogg,oga,opus"),
    ("audio/wav", "wav"),
    ("audio/webm", "webm"),
    ("audio/x-m4a", "m4a"),
    ("image/avif", "avif"),
    ("image/gif", "gif"),
    ("image/jpeg", "jpeg,jpg,jpe"),
    ("image/jxl", "jxl"),
    ("image/png", "png"),
    ("image/apng", "png,apng"),
    ("image/svg+xml", "svg,svgz"),
    ("image/webp", "webp"),
    ("multipart/related", "mht,mhtml"),
    ("text/css", "css"),
    ("text/html", "html,htm,shtml,shtm"),
    ("text/javascript", "js,mjs"),
    ("text/xml", "xml"),
    ("video/matroska", "mkv"),
    ("video/mp4", "mp4,m4v"),
    ("video/ogg", "ogv,ogm"),
    ("text/csv", "csv"),
    ("image/x-icon", "ico"),
    ("application/epub+zip", "epub"),
    ("application/font-woff", "woff"),
    ("application/gzip", "gz,tgz"),
    ("application/javascript", "js"),
    ("application/json", "json"),
    ("application/msword", "doc,dot"),
    ("application/octet-stream", "bin,exe,com"),
    ("application/pdf", "pdf"),
    ("application/pkcs7-mime", "p7m,p7c,p7z"),
    ("application/pkcs7-signature", "p7s"),
    ("application/postscript", "ps,eps,ai"),
    ("application/rdf+xml", "rdf"),
    ("application/rss+xml", "rss"),
    ("application/rtf", "rtf"),
    ("application/vnd.android.package-archive", "apk"),
    ("application/vnd.mozilla.xul+xml", "xul"),
    ("application/vnd.ms-excel", "xls"),
    ("application/vnd.ms-powerpoint", "ppt"),
    (
        "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        "pptx",
    ),
    (
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "xlsx",
    ),
    (
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        "docx",
    ),
    ("application/x-gzip", "gz,tgz"),
    ("application/x-mpegurl", "m3u8"),
    ("application/x-shockwave-flash", "swf,swl"),
    ("application/x-tar", "tar"),
    ("application/x-x509-ca-cert", "cer,crt"),
    ("application/zip", "zip"),
    ("audio/webm", "weba"),
    ("image/bmp", "bmp"),
    ("image/jpeg", "jfif,pjpeg,pjp"),
    ("image/tiff", "tiff,tif"),
    ("image/vnd.microsoft.icon", "ico"),
    ("image/x-png", "png"),
    ("image/x-xbitmap", "xbm"),
    ("message/rfc822", "eml"),
    ("text/calendar", "ics"),
    ("text/html", "ehtml"),
    ("text/markdown", "md"),
    ("text/plain", "txt,text"),
    ("text/vtt", "vtt"),
    ("text/x-sh", "sh"),
    ("text/xml", "xsl,xbl,xslt"),
    ("video/mpeg", "mpeg,mpg,mpe"),
];
pub(crate) fn GetWellKnownMimeTypeFromExtension(extension: &str) -> &'static str {
    MAPPINGS
        .iter()
        .find(|(_, extensions)| {
            extensions
                .split(',')
                .any(|ext| ext.eq_ignore_ascii_case(extension))
        })
        .map_or("", |(mime, _)| *mime)
}
