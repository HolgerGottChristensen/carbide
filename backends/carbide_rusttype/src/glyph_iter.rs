use crate::{Font, IntoGlyphId};
use crate::glyph::Glyph;

#[derive(Clone)]
pub struct GlyphIter<'a, 'font, I: Iterator>
where
    I::Item: IntoGlyphId,
{
    pub(crate) font: &'a Font<'font>,
    pub(crate) itr: I,
}

impl<'a, 'font, I> Iterator for GlyphIter<'a, 'font, I>
where
    I: Iterator,
    I::Item: IntoGlyphId,
{
    type Item = Glyph<'font>;

    fn next(&mut self) -> Option<Glyph<'font>> {
        self.itr.next().map(|c| self.font.glyph(c))
    }
}
