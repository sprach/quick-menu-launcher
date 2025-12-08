pub struct LocalizedStrings {
    pub edit_environment: String,
    pub reload: String,
    pub exit: String,
    pub warning_title: String,
    pub warning_msg: String,
}

impl LocalizedStrings {
    pub fn new(locale: &str) -> Self {
        match locale {
            "en" => Self {
                edit_environment: "Edit Environment".to_string(),
                reload: "Reload Config".to_string(),
                exit: "Exit".to_string(),
                warning_title: "Warning".to_string(),
                warning_msg: "Another instance is already running.".to_string(),
            },
            "ja" => Self {
                edit_environment: "環境編集".to_string(),
                reload: "設定再読み込み".to_string(),
                exit: "終了".to_string(),
                warning_title: "警告".to_string(),
                warning_msg: "すでに実行中です。".to_string(),
            },
            _ => Self { // Default to ko
                edit_environment: "환경 편집".to_string(),
                reload: "환경 다시 읽기".to_string(),
                exit: "종료".to_string(),
                warning_title: "경고".to_string(),
                warning_msg: "이미 실행 중입니다.".to_string(),
            },
        }
    }
}
