// SPDX-License-Identifier: GPL-3.0-or-later
//! `xtask generate-icons`: the vendored Lucide snapshot becomes Slint globals
//! of named pure functions, one per icon, so the closed icon set is checked by
//! the Slint compiler and an unused icon costs nothing in a binary. `usvg`
//! normalizes every SVG shape (circle, rect, line, ellipse, polyline, polygon,
//! arcs) into absolute move, line, curve and close commands; the generator
//! never parses path syntax itself. `xtask contracts` regenerates in memory and
//! fails when the files on disk differ.
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::{Path, PathBuf};
use usvg::tiny_skia_path::PathSegment;

/// Where the generated files live, relative to the kit UI directory.
const OUTPUT_DIRECTORY: &str = "icons/lucide";
/// The global holding every icon once.
const ALL_FILE: &str = "all.slint";
const ALL_GLOBAL: &str = "LucideIcons";
/// Coordinates keep this many decimals; Lucide draws on a 24-unit grid, so a
/// thousandth of a unit is far below one device pixel at any icon size.
const DECIMALS: usize = 3;

/// A kebab-case identifier from the snapshot (icon or category file stem).
/// Lucide names are valid Slint identifiers by construction; anything else is
/// rejected rather than rewritten, so a generated name is always the upstream
/// name.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct KebabName(String);

impl KebabName {
    fn parse(value: &str) -> Result<Self, String> {
        let valid = value.split('-').enumerate().all(|(index, part)| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                && (index > 0 || part.as_bytes()[0].is_ascii_lowercase())
        });
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(format!("`{value}` is not a kebab-case Slint identifier"))
        }
    }
    /// `food-beverage` becomes `FoodBeverage`.
    fn pascal(&self) -> String {
        self.0
            .split('-')
            .map(|part| {
                let mut chars = part.chars();
                chars.next().map_or_else(String::new, |first| {
                    first.to_ascii_uppercase().to_string() + chars.as_str()
                })
            })
            .collect()
    }
}

impl fmt::Display for KebabName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// `VERSION`: the release this snapshot was copied from.
struct Provenance {
    version: String,
    commit: String,
}

impl Provenance {
    fn parse(source: &str, directory_name: &str) -> Result<Self, String> {
        let mut fields = BTreeMap::new();
        for line in source.lines().filter(|line| !line.trim().is_empty()) {
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("VERSION: `{line}` is not `key=value`"))?;
            if fields.insert(key.trim(), value.trim()).is_some() {
                return Err(format!("VERSION: duplicate key `{key}`"));
            }
        }
        let mut take = |key: &str| {
            fields
                .remove(key)
                .filter(|value| !value.is_empty())
                .map(str::to_owned)
                .ok_or_else(|| format!("VERSION: missing `{key}`"))
        };
        let provenance = Self {
            version: take("version")?,
            commit: take("commit")?,
        };
        take("source")?;
        take("sha256")?;
        if let Some(key) = fields.keys().next() {
            return Err(format!("VERSION: unknown key `{key}`"));
        }
        if provenance.version != directory_name {
            return Err(format!(
                "VERSION names {} but the snapshot directory is {directory_name}",
                provenance.version
            ));
        }
        if provenance.commit.len() != 40
            || !provenance.commit.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(format!(
                "VERSION: `{}` is not a full commit hash",
                provenance.commit
            ));
        }
        Ok(provenance)
    }
}

/// The fields of `icons/<name>.json` the generator reads.
#[derive(Deserialize)]
struct IconMetadata {
    categories: Vec<String>,
    #[serde(default)]
    deprecated: bool,
    #[serde(rename = "deprecationReason")]
    deprecation_reason: Option<String>,
}

/// The fields of `categories/<id>.json` the generator reads.
#[derive(Deserialize)]
struct CategoryMetadata {
    title: String,
    description: String,
}

/// Upstream lifecycle of an icon. Lucide keeps deprecated icons in a release
/// until it removes them, so they stay in the generated set, marked.
enum Lifecycle {
    Current,
    Deprecated { reason: String },
}

/// Path commands on a square view box, painted by a round-capped stroke.
struct Glyph {
    commands: String,
    viewbox: f32,
    stroke: f32,
}

struct Icon {
    name: KebabName,
    categories: Vec<KebabName>,
    lifecycle: Lifecycle,
    glyph: Glyph,
}

struct Category {
    title: String,
    description: String,
}

struct Snapshot {
    /// Path relative to the repository root, for headers and messages.
    relative: String,
    provenance: Provenance,
    categories: BTreeMap<KebabName, Category>,
    icons: Vec<Icon>,
}

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("xtask lives inside the repository")
        .to_path_buf()
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Sorted file stems with the given extension in `directory`.
fn stems(directory: &Path, extension: &str) -> Result<Vec<String>, String> {
    let mut stems = Vec::new();
    for entry in
        std::fs::read_dir(directory).map_err(|e| format!("{}: {e}", directory.display()))?
    {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.extension().and_then(|e| e.to_str()) == Some(extension) {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| format!("{}: non UTF-8 file name", path.display()))?;
            stems.push(stem.to_owned());
        }
    }
    stems.sort();
    Ok(stems)
}

fn snapshot_directory(root: &Path) -> Result<(PathBuf, String), String> {
    let vendor = root.join("vendor/lucide");
    let mut versions = Vec::new();
    for entry in std::fs::read_dir(&vendor).map_err(|e| format!("{}: {e}", vendor.display()))? {
        let path = entry.map_err(|e| e.to_string())?.path();
        if path.is_dir() {
            versions.push(path);
        }
    }
    match versions.as_slice() {
        [only] => {
            let name = only
                .file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| format!("{}: non UTF-8 directory name", only.display()))?
                .to_owned();
            Ok((only.clone(), name))
        }
        [] => Err(format!("{}: no Lucide snapshot", vendor.display())),
        [_, _, ..] => Err(format!(
            "{}: keep exactly one Lucide snapshot, found {}",
            vendor.display(),
            versions.len()
        )),
    }
}

fn load_snapshot(root: &Path) -> Result<Snapshot, String> {
    let (directory, version) = snapshot_directory(root)?;
    let provenance = Provenance::parse(&read(&directory.join("VERSION"))?, &version)?;
    // The notices travel with the snapshot; a copy without them is unusable.
    read(&directory.join("LICENSE"))?;
    let mut categories = BTreeMap::new();
    for stem in stems(&directory.join("categories"), "json")? {
        let id = KebabName::parse(&stem)?;
        let path = directory.join("categories").join(format!("{stem}.json"));
        let metadata: CategoryMetadata =
            serde_json::from_str(&read(&path)?).map_err(|e| format!("{}: {e}", path.display()))?;
        categories.insert(
            id,
            Category {
                title: metadata.title,
                description: metadata.description,
            },
        );
    }
    let icon_directory = directory.join("icons");
    let svg_stems = stems(&icon_directory, "svg")?;
    let json_stems = stems(&icon_directory, "json")?;
    if svg_stems != json_stems {
        let svg: BTreeSet<_> = svg_stems.iter().collect();
        let json: BTreeSet<_> = json_stems.iter().collect();
        return Err(format!(
            "icons without metadata: {:?}; metadata without icon: {:?}",
            svg.difference(&json).collect::<Vec<_>>(),
            json.difference(&svg).collect::<Vec<_>>()
        ));
    }
    let mut icons = Vec::with_capacity(svg_stems.len());
    for stem in &svg_stems {
        let name = KebabName::parse(stem)?;
        let json_path = icon_directory.join(format!("{stem}.json"));
        let metadata: IconMetadata = serde_json::from_str(&read(&json_path)?)
            .map_err(|e| format!("{}: {e}", json_path.display()))?;
        let lifecycle = match (metadata.deprecated, metadata.deprecation_reason) {
            (false, None) => Lifecycle::Current,
            (true, Some(reason)) => Lifecycle::Deprecated { reason },
            (true, None) => {
                return Err(format!("{stem}: deprecated without a deprecationReason"));
            }
            (false, Some(_)) => {
                return Err(format!("{stem}: deprecationReason on a current icon"));
            }
        };
        let mut icon_categories = Vec::new();
        for category in &metadata.categories {
            let id = KebabName::parse(category)?;
            if !categories.contains_key(&id) {
                return Err(format!("{stem}: unknown category `{category}`"));
            }
            icon_categories.push(id);
        }
        if icon_categories.is_empty() {
            return Err(format!("{stem}: no category"));
        }
        let svg_path = icon_directory.join(format!("{stem}.svg"));
        let glyph = glyph(&read(&svg_path)?).map_err(|e| format!("{}: {e}", svg_path.display()))?;
        icons.push(Icon {
            name,
            categories: icon_categories,
            lifecycle,
            glyph,
        });
    }
    let relative = directory
        .strip_prefix(root)
        .map_err(|e| e.to_string())?
        .to_string_lossy()
        .replace('\\', "/");
    Ok(Snapshot {
        relative,
        provenance,
        categories,
        icons,
    })
}

/// SVG shape elements, counted in the source so a shape `usvg` dropped is an
/// error instead of a silently missing stroke.
const SHAPES: [&str; 7] = [
    "path", "circle", "rect", "line", "ellipse", "polyline", "polygon",
];

fn number(value: f32) -> String {
    let text = format!("{value:.DECIMALS$}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" || text.is_empty() {
        "0".to_owned()
    } else {
        text.to_owned()
    }
}

fn point(out: &mut String, point: usvg::tiny_skia_path::Point) {
    out.push(' ');
    out.push_str(&number(point.x));
    out.push(' ');
    out.push_str(&number(point.y));
}

fn glyph(source: &str) -> Result<Glyph, String> {
    let document = usvg::roxmltree::Document::parse(source).map_err(|e| e.to_string())?;
    let shapes = document
        .descendants()
        .filter(|node| SHAPES.contains(&node.tag_name().name()))
        .count();
    let tree =
        usvg::Tree::from_str(source, &usvg::Options::default()).map_err(|e| e.to_string())?;
    let size = tree.size();
    if size.width() != size.height() {
        return Err(format!(
            "view box {}x{} is not square",
            size.width(),
            size.height()
        ));
    }
    let mut paths = Vec::new();
    collect_paths(tree.root(), &mut paths)?;
    if paths.len() != shapes {
        return Err(format!("{shapes} shapes became {} paths", paths.len()));
    }
    let mut stroke_width = None;
    let mut commands = String::new();
    for path in paths {
        if !path.is_visible() {
            return Err("invisible shape".into());
        }
        if !path.abs_transform().is_identity() {
            return Err("transformed shape".into());
        }
        let stroke = path.stroke().ok_or("unstroked shape")?;
        if !matches!(stroke.linecap(), usvg::LineCap::Round)
            || !matches!(stroke.linejoin(), usvg::LineJoin::Round)
        {
            return Err("stroke caps and joins must be round".into());
        }
        let width = stroke.width().get();
        match stroke_width {
            None => stroke_width = Some(width),
            Some(first) if first == width => {}
            Some(first) => return Err(format!("stroke widths {first} and {width} differ")),
        }
        // KitIcon paints strokes only. A filled shape is accepted when the
        // round stroke already covers it (Lucide's dots), so both renderings
        // are the same disc.
        if path.fill().is_some() {
            let bounds = path.data().bounds();
            if bounds.width() > width || bounds.height() > width {
                return Err("filled shape larger than its stroke".into());
            }
        }
        for segment in path.data().segments() {
            if !commands.is_empty() {
                commands.push(' ');
            }
            match segment {
                PathSegment::MoveTo(p) => {
                    commands.push('M');
                    point(&mut commands, p);
                }
                PathSegment::LineTo(p) => {
                    commands.push('L');
                    point(&mut commands, p);
                }
                PathSegment::QuadTo(p1, p) => {
                    commands.push('Q');
                    point(&mut commands, p1);
                    point(&mut commands, p);
                }
                PathSegment::CubicTo(p1, p2, p) => {
                    commands.push('C');
                    point(&mut commands, p1);
                    point(&mut commands, p2);
                    point(&mut commands, p);
                }
                PathSegment::Close => commands.push('Z'),
            }
        }
    }
    let stroke = stroke_width.ok_or("no shape")?;
    Ok(Glyph {
        commands,
        viewbox: size.width(),
        stroke,
    })
}

fn collect_paths<'a>(group: &'a usvg::Group, out: &mut Vec<&'a usvg::Path>) -> Result<(), String> {
    for node in group.children() {
        match node {
            usvg::Node::Group(group) => collect_paths(group, out)?,
            usvg::Node::Path(path) => out.push(path),
            usvg::Node::Image(_) => return Err("embedded image".into()),
            usvg::Node::Text(_) => return Err("text element".into()),
        }
    }
    Ok(())
}

fn header(snapshot: &Snapshot, summary: &str) -> String {
    let Provenance { version, commit } = &snapshot.provenance;
    format!(
        "// Generated by `cargo run -p xtask -- generate-icons` from {path}\n\
         // (Lucide {version}, commit {commit}). Do not edit: regenerate instead.\n\
         //\n\
         // Lucide icons: ISC License, Copyright (c) 2026 Lucide Icons and Contributors.\n\
         // Icons derived from Feather: MIT License, Copyright (c) 2013-present Cole Bemis.\n\
         // Full notices: THIRD-PARTY-NOTICES.md and {path}/LICENSE.\n\
         //\n\
         // {summary}\n\
         import {{ IconGlyph }} from \"../glyph.slint\";\n",
        path = snapshot.relative,
    )
}

fn escape(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

fn function(out: &mut String, icon: &Icon) {
    if let Lifecycle::Deprecated { reason } = &icon.lifecycle {
        out.push_str(&format!("    // Deprecated by Lucide ({reason}).\n"));
    }
    let Glyph {
        commands,
        viewbox,
        stroke,
    } = &icon.glyph;
    out.push_str(&format!(
        "    public pure function {}() -> IconGlyph {{ return {{ commands: \"{}\", viewbox: {}, stroke: {} }}; }}\n",
        icon.name,
        escape(commands),
        number(*viewbox),
        number(*stroke),
    ));
}

/// Every generated file, by file name, in a deterministic order.
fn render(snapshot: &Snapshot) -> BTreeMap<String, String> {
    let mut files = BTreeMap::new();
    let mut all = header(
        snapshot,
        &format!(
            "{ALL_GLOBAL}: all {} icons. Category globals hold the same glyphs.",
            snapshot.icons.len()
        ),
    );
    all.push_str(&format!("export global {ALL_GLOBAL} {{\n"));
    for icon in &snapshot.icons {
        function(&mut all, icon);
    }
    all.push_str("}\n");
    files.insert(ALL_FILE.to_owned(), all);
    for (id, category) in &snapshot.categories {
        let members: Vec<&Icon> = snapshot
            .icons
            .iter()
            .filter(|icon| icon.categories.contains(id))
            .collect();
        let global = format!("Lucide{}", id.pascal());
        let mut file = header(
            snapshot,
            &format!(
                "{global}: {} ({} icons). {}",
                category.title,
                members.len(),
                category.description
            ),
        );
        file.push_str(&format!("export global {global} {{\n"));
        for icon in members {
            function(&mut file, icon);
        }
        file.push_str("}\n");
        files.insert(format!("{id}.slint"), file);
    }
    files
}

fn output_directory() -> PathBuf {
    kit_build::ui_directory().join(OUTPUT_DIRECTORY)
}

fn existing_files(directory: &Path) -> Result<BTreeSet<String>, String> {
    if !directory.exists() {
        return Ok(BTreeSet::new());
    }
    Ok(stems(directory, "slint")?
        .into_iter()
        .map(|stem| format!("{stem}.slint"))
        .collect())
}

pub fn generate() -> Result<(), String> {
    let files = render(&load_snapshot(&repository_root())?);
    let directory = output_directory();
    std::fs::create_dir_all(&directory).map_err(|e| format!("{}: {e}", directory.display()))?;
    for stale in existing_files(&directory)? {
        if !files.contains_key(&stale) {
            let path = directory.join(stale);
            std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        }
    }
    for (name, content) in &files {
        let path = directory.join(name);
        std::fs::write(&path, content).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    println!(
        "generate-icons: {} files written to {}",
        files.len(),
        directory.display()
    );
    Ok(())
}

pub fn verify() -> Result<(), String> {
    let files = render(&load_snapshot(&repository_root())?);
    let directory = output_directory();
    let mut problems = Vec::new();
    for name in existing_files(&directory)? {
        if !files.contains_key(&name) {
            problems.push(format!("{name} is not generated by the snapshot"));
        }
    }
    for (name, content) in &files {
        let path = directory.join(name);
        match std::fs::read_to_string(&path) {
            Ok(existing) if existing == *content => {}
            Ok(_) => problems.push(format!("{name} differs from the snapshot")),
            Err(error) => problems.push(format!("{name}: {error}")),
        }
    }
    if problems.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "generated icons in {} are stale; run `cargo run -p xtask -- generate-icons`:\n  {}",
            directory.display(),
            problems.join("\n  ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHAPES_SVG: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
  <path d="M12 19V5" />
  <line x1="5" y1="12" x2="19" y2="12" />
  <circle cx="12" cy="12" r="10" />
  <circle cx="12" cy="12" r=".5" fill="currentColor" />
</svg>"#;

    #[test]
    fn shapes_flatten_to_absolute_commands_with_the_source_stroke() {
        let glyph = glyph(SHAPES_SVG).unwrap();
        assert_eq!(glyph.viewbox, 24.);
        assert_eq!(glyph.stroke, 2.);
        assert!(
            glyph
                .commands
                .starts_with("M 12 19 L 12 5 M 5 12 L 19 12 M "),
            "{}",
            glyph.commands
        );
        // Circles become closed cubic curves, never arcs or relative commands.
        assert_eq!(glyph.commands.matches('C').count(), 8);
        assert_eq!(glyph.commands.matches('Z').count(), 2);
        assert!(!glyph.commands.contains(['A', 'a', 'h', 'v', 'H', 'V']));
    }

    #[test]
    fn unsupported_paint_is_rejected() {
        let filled = SHAPES_SVG.replace("r=\".5\"", "r=\"4\"");
        assert_eq!(
            glyph(&filled).err().as_deref(),
            Some("filled shape larger than its stroke")
        );
        let butt = SHAPES_SVG.replace("stroke-linecap=\"round\"", "stroke-linecap=\"butt\"");
        assert_eq!(
            glyph(&butt).err().as_deref(),
            Some("stroke caps and joins must be round")
        );
        let moved = SHAPES_SVG.replace("<line ", "<line transform=\"translate(1 0)\" ");
        assert_eq!(glyph(&moved).err().as_deref(), Some("transformed shape"));
    }

    #[test]
    fn numbers_are_short_and_stable() {
        assert_eq!(number(12.0), "12");
        assert_eq!(number(0.5), "0.5");
        assert_eq!(number(-0.0001), "0");
        assert_eq!(number(5.523_000_4), "5.523");
    }

    #[test]
    fn names_stay_upstream_kebab_case() {
        assert_eq!(
            KebabName::parse("food-beverage").unwrap().pascal(),
            "FoodBeverage"
        );
        assert!(KebabName::parse("dice-1").is_ok());
        for invalid in ["1st", "Arrow", "arrow--up", "arrow-", "arrow_up"] {
            assert!(KebabName::parse(invalid).is_err(), "{invalid}");
        }
    }

    #[test]
    fn version_file_is_strict() {
        let ok =
            "version=1.0.0\ncommit=be09619c1c1124ad09c19fc3e87ee3b939409bbd\nsource=s\nsha256=h\n";
        assert!(Provenance::parse(ok, "1.0.0").is_ok());
        assert!(Provenance::parse(ok, "1.0.1").is_err());
        assert!(Provenance::parse(&ok.replace("sha256=h\n", ""), "1.0.0").is_err());
        assert!(Provenance::parse(&format!("{ok}extra=1\n"), "1.0.0").is_err());
    }

    #[test]
    fn the_vendored_snapshot_generates_every_icon() {
        let snapshot = load_snapshot(&repository_root()).unwrap();
        let files = render(&snapshot);
        assert_eq!(files.len(), snapshot.categories.len() + 1);
        let all = &files[ALL_FILE];
        assert_eq!(
            all.matches("public pure function ").count(),
            snapshot.icons.len()
        );
        assert!(all.contains(
            "public pure function arrow-up() -> IconGlyph { return { commands: \"M 5 12 L 12 5 L 19 12 M 12 19 L 12 5\", viewbox: 24, stroke: 2 }; }"
        ));
        // Every icon appears in each of its categories.
        let memberships: usize = snapshot.icons.iter().map(|i| i.categories.len()).sum();
        let listed: usize = files
            .iter()
            .filter(|(name, _)| name.as_str() != ALL_FILE)
            .map(|(_, file)| file.matches("public pure function ").count())
            .sum();
        assert_eq!(listed, memberships);
    }
}
