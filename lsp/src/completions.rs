pub const HTTP_METHODS: &[(&str, &str)] = &[
    ("GET", "Retrieve data"),
    ("POST", "Submit data"),
    ("PUT", "Update/replace data"),
    ("PATCH", "Partial update"),
    ("DELETE", "Remove data"),
    ("HEAD", "Get headers only"),
    ("OPTIONS", "Get allowed methods"),
    ("CONNECT", "Establish tunnel"),
    ("TRACE", "Loop-back test"),
];

pub const HEADER_NAMES: &[(&str, &str)] = &[
    ("Content-Type", "application/json"),
    ("Content-Encoding", "gzip"),
    ("Authorization", "Bearer "),
    ("Accept", "application/json"),
    ("Accept-Encoding", "gzip, deflate"),
    ("Accept-Language", "en-US"),
    ("Cache-Control", "no-cache"),
    ("Connection", "keep-alive"),
    ("Cookie", ""),
    ("Host", ""),
    ("Origin", ""),
    ("Referer", ""),
    ("User-Agent", "RestClient/1.0"),
    ("Transfer-Encoding", "chunked"),
    ("X-Request-ID", ""),
    ("X-API-Key", ""),
    ("X-Content-Type-Options", "nosniff"),
    ("X-Frame-Options", "DENY"),
    ("Access-Control-Allow-Origin", "*"),
    (
        "Access-Control-Allow-Methods",
        "GET, POST, PUT, DELETE, OPTIONS",
    ),
    (
        "Access-Control-Allow-Headers",
        "Content-Type, Authorization",
    ),
    ("If-None-Match", ""),
    ("If-Modified-Since", ""),
];

pub const AUTH_SCHEMES: &[&str] = &["Bearer ", "Basic ", "Token "];

pub fn header_values(name: &str) -> &'static [&'static str] {
    match name {
        "content-type" => &[
            "application/json",
            "application/xml",
            "application/x-www-form-urlencoded",
            "multipart/form-data",
            "text/plain",
            "text/html",
            "text/csv",
            "application/octet-stream",
        ],
        "accept" => &[
            "application/json",
            "application/xml",
            "text/html",
            "text/plain",
            "*/*",
        ],
        "cache-control" => &[
            "no-cache",
            "no-store",
            "max-age=0",
            "max-age=3600",
            "must-revalidate",
            "public",
            "private",
        ],
        "connection" => &["keep-alive", "close"],
        "accept-encoding" => &[
            "gzip, deflate",
            "gzip, deflate, br",
            "gzip",
            "br",
            "identity",
        ],
        "accept-language" => &[
            "en-US",
            "en-US,en;q=0.9",
            "en-GB",
            "fr-FR",
            "de-DE",
            "zh-CN",
            "ja-JP",
            "*",
        ],
        "content-encoding" => &["gzip", "deflate", "br", "identity"],
        "transfer-encoding" => &["chunked", "compress", "deflate", "gzip"],
        "x-content-type-options" => &["nosniff"],
        "x-frame-options" => &["DENY", "SAMEORIGIN"],
        "access-control-allow-origin" => &["*"],
        "access-control-allow-methods" => {
            &["GET, POST, PUT, DELETE, OPTIONS", "GET, POST, OPTIONS", "*"]
        }
        "access-control-allow-headers" => &["Content-Type, Authorization", "Content-Type", "*"],
        _ => &[],
    }
}
