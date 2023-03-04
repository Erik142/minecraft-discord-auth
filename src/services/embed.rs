use serenity::utils::Colour;

#[derive(Debug)]
pub struct EmbedData {
    pub title: Option<String>,
    pub description: Option<String>,
    pub colour: Option<Colour>,
}
