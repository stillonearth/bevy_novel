use bevy::{
    asset::{io::Reader, AssetLoader, LoadContext},
    prelude::*,
};
use renpy_parser::{parse_scenario_from_string, parsers::AST};
use thiserror::Error;

#[derive(Default)]
pub struct RpyAssetLoader;

#[derive(Asset, TypePath, Debug, Deref, DerefMut)]
pub struct Rpy(pub Vec<AST>);

/// Possible errors that can be produced by [`RpyAssetLoader`]
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum RpyAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could not load file: {0}")]
    Io(#[from] std::io::Error),
}

impl AssetLoader for RpyAssetLoader {
    type Asset = Rpy;
    type Settings = ();
    type Error = RpyAssetLoaderError;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &(),
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;

        let content = std::str::from_utf8(&bytes).unwrap();
        let (ast, _) = parse_scenario_from_string(content, "_").unwrap();

        Ok(Rpy(ast))
    }
}
