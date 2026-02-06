use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Framework {
    NextJs,
    ViteReact,
    SvelteKit,
    Nuxt,
}

impl Framework {
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "nextjs" => Some(Framework::NextJs),
            "vite_react" => Some(Framework::ViteReact),
            "sveltekit" => Some(Framework::SvelteKit),
            "nuxt" => Some(Framework::Nuxt),
            _ => None,
        }
    }

    pub fn to_db(&self) -> &str {
        match self {
            Framework::NextJs => "nextjs",
            Framework::ViteReact => "vite_react",
            Framework::SvelteKit => "sveltekit",
            Framework::Nuxt => "nuxt",
        }
    }
}

impl fmt::Display for Framework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Framework::NextJs => write!(f, "Next.js"),
            Framework::ViteReact => write!(f, "Vite/React"),
            Framework::SvelteKit => write!(f, "SvelteKit"),
            Framework::Nuxt => write!(f, "Nuxt"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Platform {
    Vercel,
    Netlify,
    Railway,
}

impl Platform {
    pub fn from_db(s: &str) -> Option<Self> {
        match s {
            "vercel" => Some(Platform::Vercel),
            "netlify" => Some(Platform::Netlify),
            "railway" => Some(Platform::Railway),
            _ => None,
        }
    }

    pub fn to_db(&self) -> &str {
        match self {
            Platform::Vercel => "vercel",
            Platform::Netlify => "netlify",
            Platform::Railway => "railway",
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Vercel => write!(f, "Vercel"),
            Platform::Netlify => write!(f, "Netlify"),
            Platform::Railway => write!(f, "Railway"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub framework: Option<Framework>,
    pub platform: Option<Platform>,
    pub framework_confidence: u8,
    pub platform_confidence: u8,
    pub signals: Vec<String>,
}
