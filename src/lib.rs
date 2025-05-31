pub mod lexicon;
pub mod com {
    pub mod macroblog {
        pub mod blog {
            pub mod post {
                use serde::{Deserialize, Serialize};
                use atrium_api::types::string::Datetime;

                #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
                #[serde(rename_all = "camelCase")]
                pub struct Record {
                    #[serde(rename = "$type")]
                    pub r#type: String,
                    pub created_at: Datetime,
                    pub title: String,
                    pub text: String,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub tags: Option<Vec<String>>,
                }

                #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
                #[serde(rename_all = "camelCase")]
                pub struct RecordData {
                    pub title: String,
                    pub text: String,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    pub tags: Option<Vec<String>>,
                }

                impl From<RecordData> for Record {
                    fn from(data: RecordData) -> Self {
                        Record {
                            r#type: "com.macroblog.blog.post".to_string(),
                            created_at: Datetime::now(),
                            title: data.title,
                            text: data.text,
                            tags: data.tags,
                        }
                    }
                }
            }
        }
    }
} 