use gpui::{App, AssetSource};

pub fn init(cx: &mut App) -> gpui::Result<()> {
    load_fonts(cx)?;
    Ok(())
}

#[derive(rust_embed::RustEmbed)]
#[folder = "./assets"]
#[include = "fonts/**/*.ttf"]
#[include = "icons/**/*.svg"]
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if path.is_empty() {
            return Ok(None);
        }

        Self::get(path)
            .map(|f| Some(f.data))
            .ok_or_else(|| anyhow::anyhow!("could not find asset at path \"{path}\""))
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
        Ok(Self::iter()
            .filter_map(|p| p.starts_with(path).then(|| p.into()))
            .collect())
    }
}

fn load_fonts(cx: &mut App) -> gpui::Result<()> {
    let font_paths = cx.asset_source().list("fonts")?;

    let embedded_fonts = font_paths
        .into_iter()
        .filter(|p| p.ends_with(".ttf"))
        .map(|p| cx.asset_source().load(&p))
        .filter_map(|res| res.ok().flatten())
        .collect::<Vec<_>>();

    log::debug!("Loaded {} fonts", embedded_fonts.len(),);
    cx.text_system().add_fonts(embedded_fonts)?;

    Ok(())
}
