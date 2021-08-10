use syn;

// An iterator yielding all carbide attributes in the given attributes.
pub struct CarbideAttrs<I> {
    attrs: I,
}

pub fn carbide_attrs<'a, I>(attrs: I) -> CarbideAttrs<I::IntoIter>
    where
        I: IntoIterator<Item=&'a syn::Attribute>,
{
    CarbideAttrs {
        attrs: attrs.into_iter(),
    }
}

impl<'a, I> Iterator for CarbideAttrs<I>
    where
        I: Iterator<Item=&'a syn::Attribute>,
{
    type Item = Vec<syn::NestedMeta>;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(attr) = self.attrs.next() {
            if let Ok(_meta) = attr.parse_meta() {
                if let &syn::Meta::List(ref _metalist) = &_meta {
                    if _metalist.path.is_ident("carbide") {
                        let j = _metalist
                            .nested
                            .clone()
                            .into_pairs()
                            .map(|pair| pair.into_value())
                            .collect::<Vec<syn::NestedMeta>>();
                        return Some(j);
                    }
                }
            }
        }
        None
    }
}
