//! The list's model: one card per mod, and which cards a search, a filter and a sort show.
//! Nothing here needs the game; the game half fills the cards from the installed mods.

use crate::config::Sort;

/// Where a card stands.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Status {
    /// Loaded and running.
    Running,
    /// Installed but not loaded (turned off in the launcher).
    Off,
    /// It failed to load: why.
    Failed(String),
}

/// A key a mod adds: its label's translation key and the key it is on now.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KeyInfo {
    pub label: String,
    pub key: String,
}

/// One mod as SunnyMods shows it.
#[derive(Clone, Debug, PartialEq)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub version: String,
    pub authors: Vec<String>,
    pub credits: String,
    pub license: String,
    pub license_url: Option<String>,
    pub homepage: Option<String>,
    pub issues: Option<String>,
    pub description: String,
    /// Its place in the game's installed list.
    pub index: Option<usize>,
    /// How many settings its page has.
    pub settings: usize,
    pub keys: Vec<KeyInfo>,
    pub status: Status,
    /// Loading notices: translation key and arguments.
    pub warnings: Vec<(String, Vec<String>)>,
    /// It has an `icon.png`.
    pub has_icon: bool,
    /// Loaded with `--dev-mod`.
    pub dev: bool,
    /// A native mod: it can have a settings page.
    pub native: bool,
}

impl Card {
    /// Its settings page can open.
    pub fn has_settings(&self) -> bool {
        self.native && self.index.is_some() && (self.settings > 0 || !self.keys.is_empty())
    }
    pub fn authors_line(&self) -> String {
        self.authors.join(", ")
    }
    /// The first sentence (or line) of its description, for its row.
    pub fn summary(&self) -> &str {
        let d = self.description.trim();
        let line = d.lines().next().unwrap_or("");
        match line.find(". ") {
            Some(i) => &line[..=i],
            None => line,
        }
    }
    /// Whether the search's words all appear in its name, id, authors or description.
    pub fn matches(&self, query: &str) -> bool {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return true;
        }
        let hay = format!(
            "{}\n{}\n{}\n{}\n{}",
            self.name.to_lowercase(),
            self.id.to_lowercase(),
            self.authors_line().to_lowercase(),
            self.description.to_lowercase(),
            self.license.to_lowercase()
        );
        query.split_whitespace().all(|word| {
            // `@name` looks only at the authors, `#id` only at the id.
            if let Some(author) = word.strip_prefix('@') {
                self.authors_line().to_lowercase().contains(author)
            } else if let Some(id) = word.strip_prefix('#') {
                self.id.to_lowercase().contains(id)
            } else {
                hay.contains(word)
            }
        })
    }
}

/// The chips over the list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Filter {
    All,
    Running,
    Settings,
    Issues,
}

impl Filter {
    pub const ALL: [Filter; 4] = [Filter::All, Filter::Running, Filter::Settings, Filter::Issues];
    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Running => "Running",
            Self::Settings => "Settings",
            Self::Issues => "Issues",
        }
    }
    pub fn lang(self) -> &'static str {
        match self {
            Self::All => "sunnymods.filter.all",
            Self::Running => "sunnymods.filter.running",
            Self::Settings => "sunnymods.filter.settings",
            Self::Issues => "sunnymods.filter.issues",
        }
    }
    pub fn keeps(self, card: &Card) -> bool {
        match self {
            Self::All => true,
            Self::Running => card.status == Status::Running,
            Self::Settings => card.has_settings(),
            Self::Issues => {
                matches!(card.status, Status::Failed(_) | Status::Off) || !card.warnings.is_empty()
            }
        }
    }
}

/// The cards shown, in order: the mods the search and filter keep, sorted.
pub fn shown(cards: &[Card], query: &str, filter: Filter, sort: Sort) -> Vec<usize> {
    let mut out: Vec<usize> = (0..cards.len())
        .filter(|&i| {
            let c = &cards[i];
            filter.keeps(c) && c.matches(query)
        })
        .collect();
    let key = |c: &Card| c.name.to_lowercase();
    out.sort_by(|&a, &b| {
        let (ca, cb) = (&cards[a], &cards[b]);
        match sort {
            Sort::NameAz => key(ca).cmp(&key(cb)),
            Sort::NameZa => key(cb).cmp(&key(ca)),
            Sort::Author => ca
                .authors_line()
                .to_lowercase()
                .cmp(&cb.authors_line().to_lowercase())
                .then_with(|| key(ca).cmp(&key(cb))),
            Sort::SettingsFirst => cb
                .has_settings()
                .cmp(&ca.has_settings())
                .then_with(|| key(ca).cmp(&key(cb))),
        }
    });
    out
}

/// Counts for the header: installed mods, running ones, ones with a problem.
pub fn counts(cards: &[Card]) -> (usize, usize, usize) {
    let mods = cards.iter();
    let (mut total, mut running, mut issues) = (0, 0, 0);
    for c in mods {
        total += 1;
        match c.status {
            Status::Running => running += 1,
            Status::Failed(_) => issues += 1,
            Status::Off => {}
        }
    }
    (total, running, issues)
}

/// A link a card can open, checked: only http(s) addresses.
pub fn safe_link(url: Option<&str>) -> Option<&str> {
    let url = url?.trim();
    (url.starts_with("https://") || url.starts_with("http://")).then_some(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(id: &str, name: &str, author: &str) -> Card {
        Card {
            id: id.into(),
            name: name.into(),
            version: "1.0.0".into(),
            authors: vec![author.into()],
            credits: String::new(),
            license: "MIT".into(),
            license_url: None,
            homepage: None,
            issues: None,
            description: format!("{name} does things. More words."),
            index: Some(0),
            settings: 0,
            keys: Vec::new(),
            status: Status::Running,
            warnings: Vec::new(),
            has_icon: false,
            dev: false,
            native: true,
        }
    }

    #[test]
    fn search_filter_and_sort() {
        let mut cards = vec![card("b", "Beta", "Zed"), card("a", "Alpha", "Amy"), card("c", "Gamma", "Amy")];
        cards[2].settings = 3;
        assert_eq!(shown(&cards, "", Filter::All, Sort::NameAz), vec![1, 0, 2]);
        assert_eq!(shown(&cards, "", Filter::All, Sort::NameZa), vec![2, 0, 1]);
        assert_eq!(shown(&cards, "", Filter::Settings, Sort::NameAz), vec![2]);
        assert_eq!(shown(&cards, "@amy", Filter::All, Sort::NameAz), vec![1, 2]);
        assert_eq!(shown(&cards, "#b", Filter::All, Sort::NameAz), vec![0]);
        assert_eq!(shown(&cards, "gamma things", Filter::All, Sort::NameAz), vec![2]);
        assert_eq!(shown(&cards, "", Filter::All, Sort::SettingsFirst)[0], 2);
        assert_eq!(counts(&cards), (3, 3, 0));
    }

    #[test]
    fn summaries_and_links() {
        let c = card("a", "Alpha", "Amy");
        assert_eq!(c.summary(), "Alpha does things.");
        assert_eq!(safe_link(Some(" https://x.y ")), Some("https://x.y"));
        assert_eq!(safe_link(Some("file:///etc")), None);
    }
}
