use std::path::PathBuf;

use serde::Deserialize;

use crate::detect::Agent;

use super::io::resolve_config_relative_path;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct SoundConfig {
    pub enabled: bool,
    /// Audio plays mp3 files on the client host; bell emits terminal bells, including over SSH.
    pub backend: SoundBackend,
    pub bell: BellPatternConfig,
    /// Optional mp3 file path used for all notification sounds.
    /// Relative paths are resolved from the config file's directory.
    pub path: Option<PathBuf>,
    /// Optional mp3 file path for "done" notifications.
    /// Relative paths are resolved from the config file's directory.
    pub done_path: Option<PathBuf>,
    /// Optional mp3 file path for "request" notifications.
    /// Relative paths are resolved from the config file's directory.
    pub request_path: Option<PathBuf>,
    pub agents: AgentSoundOverrides,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundBackend {
    #[default]
    Audio,
    Bell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct BellPatternConfig {
    /// Bells in the shared prefix; 0 disables the prefix. Range: 0–8.
    pub prefix_count: u8,
    /// Milliseconds between prefix bells. Range: 50–1000.
    pub prefix_gap_ms: u64,
    /// Milliseconds between prefix and suffix; ignored without a prefix. Range: 0–2000.
    pub separator_ms: u64,
    /// Bells in the Done suffix. Range: 1–8.
    pub done_count: u8,
    /// Bells in the Request suffix. Range: 1–8.
    pub request_count: u8,
    /// Milliseconds between suffix bells. Range: 50–1000.
    pub suffix_gap_ms: u64,
}

impl Default for BellPatternConfig {
    fn default() -> Self {
        Self {
            prefix_count: 0,
            prefix_gap_ms: 100,
            separator_ms: 600,
            done_count: 1,
            request_count: 2,
            suffix_gap_ms: 300,
        }
    }
}

impl BellPatternConfig {
    pub fn bounded(self) -> Self {
        Self {
            prefix_count: self.prefix_count.min(8),
            prefix_gap_ms: self.prefix_gap_ms.clamp(50, 1000),
            separator_ms: self.separator_ms.min(2000),
            done_count: self.done_count.clamp(1, 8),
            request_count: self.request_count.clamp(1, 8),
            suffix_gap_ms: self.suffix_gap_ms.clamp(50, 1000),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(default)]
pub struct AgentSoundOverrides {
    pub pi: AgentSoundSetting,
    pub claude: AgentSoundSetting,
    pub codex: AgentSoundSetting,
    pub gemini: AgentSoundSetting,
    pub cursor: AgentSoundSetting,
    pub devin: AgentSoundSetting,
    pub agy: AgentSoundSetting,
    pub cline: AgentSoundSetting,
    pub open_code: AgentSoundSetting,
    pub github_copilot: AgentSoundSetting,
    pub kimi: AgentSoundSetting,
    pub kiro: AgentSoundSetting,
    pub droid: AgentSoundSetting,
    pub amp: AgentSoundSetting,
    pub grok: AgentSoundSetting,
    pub hermes: AgentSoundSetting,
    pub kilo: AgentSoundSetting,
    pub qodercli: AgentSoundSetting,
    pub qwen: AgentSoundSetting,
    pub letta: AgentSoundSetting,
    pub maki: AgentSoundSetting,
    pub muse: AgentSoundSetting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentSoundSetting {
    #[default]
    Default,
    On,
    Off,
}

impl SoundConfig {
    pub fn allows(&self, agent: Option<Agent>) -> bool {
        if !self.enabled {
            return false;
        }

        !matches!(self.agents.for_agent(agent), AgentSoundSetting::Off)
    }

    pub fn path_for(&self, sound: crate::sound::Sound) -> Option<PathBuf> {
        let path = match sound {
            crate::sound::Sound::Done => self.done_path.as_ref().or(self.path.as_ref()),
            crate::sound::Sound::Request => self.request_path.as_ref().or(self.path.as_ref()),
        }?;

        Some(resolve_config_relative_path(path))
    }

    pub fn diagnostics(&self) -> Vec<String> {
        if self.backend == SoundBackend::Bell {
            return if self.bell == self.bell.bounded() {
                Vec::new()
            } else {
                vec!["ui.sound.bell: out-of-range counts or timings are clamped to the documented limits".into()]
            };
        }
        let mut diagnostics = Vec::new();
        for (field, path) in [
            ("ui.sound.path", self.path.as_ref()),
            ("ui.sound.done_path", self.done_path.as_ref()),
            ("ui.sound.request_path", self.request_path.as_ref()),
        ] {
            let Some(path) = path else {
                continue;
            };

            let resolved = resolve_config_relative_path(path);
            if resolved
                .extension()
                .and_then(|ext| ext.to_str())
                .is_none_or(|ext: &str| !ext.eq_ignore_ascii_case("mp3"))
            {
                diagnostics.push(format!(
                    "unsupported sound file format: {field} = {} resolves to {}; expected an mp3 file; using default sound",
                    path.display(),
                    resolved.display()
                ));
                continue;
            }

            if !resolved.exists() {
                diagnostics.push(format!(
                    "missing sound file: {field} = {} resolves to {}; using default sound",
                    path.display(),
                    resolved.display()
                ));
            } else if !resolved.is_file() {
                diagnostics.push(format!(
                    "invalid sound file: {field} = {} resolves to {}; using default sound",
                    path.display(),
                    resolved.display()
                ));
            }
        }
        diagnostics
    }
}

impl AgentSoundOverrides {
    pub fn for_agent(&self, agent: Option<Agent>) -> AgentSoundSetting {
        match agent {
            Some(Agent::Pi) => self.pi,
            Some(Agent::Claude) => self.claude,
            Some(Agent::Codex) => self.codex,
            Some(Agent::Gemini) => self.gemini,
            Some(Agent::Cursor) => self.cursor,
            Some(Agent::Devin) => self.devin,
            Some(Agent::Antigravity) => self.agy,
            Some(Agent::Cline) => self.cline,
            Some(Agent::Omp) => AgentSoundSetting::Default,
            Some(Agent::Mastracode) => AgentSoundSetting::Default,
            Some(Agent::OpenCode) => self.open_code,
            Some(Agent::GithubCopilot) => self.github_copilot,
            Some(Agent::Kimi) => self.kimi,
            Some(Agent::Kiro) => self.kiro,
            Some(Agent::Droid) => self.droid,
            Some(Agent::Amp) => self.amp,
            Some(Agent::Grok) => self.grok,
            Some(Agent::Hermes) => self.hermes,
            Some(Agent::Kilo) => self.kilo,
            Some(Agent::Qodercli) => self.qodercli,
            Some(Agent::Qwen) => self.qwen,
            Some(Agent::Letta) => self.letta,
            Some(Agent::Maki) => self.maki,
            Some(Agent::Muse) => self.muse,
            None => AgentSoundSetting::Default,
        }
    }
}

impl Default for SoundConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: SoundBackend::default(),
            bell: BellPatternConfig::default(),
            path: None,
            done_path: None,
            request_path: None,
            agents: AgentSoundOverrides::default(),
        }
    }
}

impl Default for AgentSoundOverrides {
    fn default() -> Self {
        Self {
            pi: AgentSoundSetting::Default,
            claude: AgentSoundSetting::Default,
            codex: AgentSoundSetting::Default,
            gemini: AgentSoundSetting::Default,
            cursor: AgentSoundSetting::Default,
            devin: AgentSoundSetting::Default,
            agy: AgentSoundSetting::Default,
            cline: AgentSoundSetting::Default,
            open_code: AgentSoundSetting::Default,
            github_copilot: AgentSoundSetting::Default,
            kimi: AgentSoundSetting::Default,
            kiro: AgentSoundSetting::Default,
            droid: AgentSoundSetting::Off,
            amp: AgentSoundSetting::Default,
            grok: AgentSoundSetting::Default,
            hermes: AgentSoundSetting::Default,
            kilo: AgentSoundSetting::Default,
            qodercli: AgentSoundSetting::Default,
            qwen: AgentSoundSetting::Default,
            letta: AgentSoundSetting::Default,
            maki: AgentSoundSetting::Default,
            muse: AgentSoundSetting::Default,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::config::{config_path, Config};

    #[test]
    fn bell_signature_config_parses_and_bounds_extreme_values() {
        let config: Config = toml::from_str(
            "[ui.sound]\nbackend='bell'\n[ui.sound.bell]\nprefix_count=2\nprefix_gap_ms=100\nseparator_ms=600\ndone_count=1\nrequest_count=3\nsuffix_gap_ms=100",
        ).unwrap();
        assert_eq!(config.ui.sound.bell.prefix_count, 2);
        assert_eq!(config.ui.sound.bell.request_count, 3);
        assert_eq!(config.ui.sound.bell.suffix_gap_ms, 100);
        assert!(config.ui.sound.diagnostics().is_empty());
        let extreme: Config = toml::from_str(
            "[ui.sound]\nbackend='bell'\n[ui.sound.bell]\nprefix_count=255\nprefix_gap_ms=0\nseparator_ms=999999\ndone_count=0\nrequest_count=255\nsuffix_gap_ms=999999",
        ).unwrap();
        let bounded = extreme.ui.sound.bell.bounded();
        assert_eq!(bounded.prefix_count, 8);
        assert_eq!(bounded.prefix_gap_ms, 50);
        assert_eq!(bounded.separator_ms, 2000);
        assert_eq!(bounded.done_count, 1);
        assert_eq!(bounded.request_count, 8);
        assert_eq!(bounded.suffix_gap_ms, 1000);
        assert!(!extreme.ui.sound.diagnostics().is_empty());
    }

    #[test]
    fn sound_backend_defaults_to_audio_and_parses_bell() {
        let existing: Config = toml::from_str("[ui.sound]\nenabled = true").unwrap();
        assert_eq!(existing.ui.sound.backend, SoundBackend::Audio);
        let bell: Config =
            toml::from_str("[ui.sound]\nbackend = 'bell'\ndone_path = 'unused.wav'").unwrap();
        assert_eq!(bell.ui.sound.backend, SoundBackend::Bell);
        assert!(bell.ui.sound.diagnostics().is_empty());
        assert!(bell.ui.sound.allows(Some(Agent::Codex)));
        assert!(!bell.ui.sound.allows(Some(Agent::Droid)));
        assert!(toml::from_str::<Config>("[ui.sound]\nbackend = 'invalid'").is_err());
        let muted: Config =
            toml::from_str("[ui.sound]\nbackend = 'bell'\nenabled = false").unwrap();
        assert!(!muted.ui.sound.allows(Some(Agent::Codex)));
    }

    #[test]
    fn sound_table_config_parses() {
        let toml = r#"
[ui.sound]
enabled = true
path = "sounds/all.mp3"
done_path = "sounds/done.mp3"
request_path = "/tmp/request.mp3"

[ui.sound.agents]
droid = "off"
claude = "on"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.ui.sound.enabled);
        assert_eq!(config.ui.sound.path, Some(PathBuf::from("sounds/all.mp3")));
        assert_eq!(
            config.ui.sound.done_path,
            Some(PathBuf::from("sounds/done.mp3"))
        );
        assert_eq!(
            config.ui.sound.request_path,
            Some(PathBuf::from("/tmp/request.mp3"))
        );
        assert_eq!(config.ui.sound.agents.droid, AgentSoundSetting::Off);
        assert_eq!(config.ui.sound.agents.claude, AgentSoundSetting::On);
        assert_eq!(config.ui.sound.agents.pi, AgentSoundSetting::Default);
        assert_eq!(config.ui.sound.agents.letta, AgentSoundSetting::Default);
    }

    #[test]
    fn letta_sound_override_is_used() {
        let overrides: AgentSoundOverrides = toml::from_str("letta = \"on\"").unwrap();
        assert_eq!(overrides.letta, AgentSoundSetting::On);
        assert_eq!(
            overrides.for_agent(Some(Agent::Letta)),
            AgentSoundSetting::On
        );
    }

    #[test]
    fn sound_path_resolution_prefers_specific_over_global() {
        let config: Config = toml::from_str(
            r#"
[ui.sound]
path = "sounds/all.mp3"
done_path = "sounds/done.mp3"
"#,
        )
        .unwrap();

        let config_root = config_path().parent().unwrap().to_path_buf();
        assert_eq!(
            config.ui.sound.path_for(crate::sound::Sound::Done),
            Some(config_root.join("sounds/done.mp3"))
        );
        assert_eq!(
            config.ui.sound.path_for(crate::sound::Sound::Request),
            Some(config_root.join("sounds/all.mp3"))
        );
    }

    #[test]
    fn missing_sound_file_produces_diagnostic() {
        let config: Config = toml::from_str(
            r#"
[ui.sound]
done_path = "sounds/missing.mp3"
"#,
        )
        .unwrap();

        let diagnostics = config.collect_diagnostics();
        assert!(diagnostics.iter().any(
            |diag| diag.contains("ui.sound.done_path") && diag.contains("using default sound")
        ));
    }

    #[test]
    fn non_mp3_sound_file_produces_diagnostic() {
        let config: Config = toml::from_str(
            r#"
[ui.sound]
path = "sounds/notification.wav"
"#,
        )
        .unwrap();

        let diagnostics = config.collect_diagnostics();
        assert!(diagnostics.iter().any(|diag| {
            diag.contains("ui.sound.path") && diag.contains("expected an mp3 file")
        }));
    }
}
