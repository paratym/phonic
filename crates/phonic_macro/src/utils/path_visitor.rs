use proc_macro2::Span;
use syn::{visit_mut::VisitMut, Ident, Path, PathSegment, Token};

pub enum CratePathVisitorMethod {
    Expand,
    Inline { strict: bool },
}

pub struct CratePathVisitor {
    pub method: CratePathVisitorMethod,
    pub crate_name: Ident,
}

impl CratePathVisitor {
    fn new(method: CratePathVisitorMethod) -> Self {
        const ENV_KEY: &str = "CARGO_CRATE_NAME";
        let msg = format!("missing env var {ENV_KEY}");
        let crate_name_str = std::env::var(ENV_KEY).expect(&msg);
        let crate_name = Ident::new(&crate_name_str, Span::call_site());

        Self { method, crate_name }
    }

    pub fn expand() -> Self {
        Self::new(CratePathVisitorMethod::Expand)
    }

    pub fn inline() -> Self {
        Self::new(CratePathVisitorMethod::Inline { strict: false })
    }

    pub fn inline_strict() -> Self {
        Self::new(CratePathVisitorMethod::Inline { strict: true })
    }
}

impl VisitMut for CratePathVisitor {
    fn visit_path_mut(&mut self, path: &mut Path) {
        let global_root = path.leading_colon.is_some();
        match (path.segments.first_mut(), &self.method) {
            (Some(PathSegment { ident, .. }), CratePathVisitorMethod::Expand)
                if *ident == Ident::from(<Token![crate]>::default()) =>
            {
                *ident = self.crate_name.clone();
                path.leading_colon = path.leading_colon.or(Some(<Token![::]>::default()));
            }

            (Some(PathSegment { ident, .. }), CratePathVisitorMethod::Inline { strict })
                if *ident == self.crate_name && (!strict || global_root) =>
            {
                *ident = <Token![crate]>::default().into();
                path.leading_colon = None;
            }

            _ => {}
        }

        path.segments
            .iter_mut()
            .for_each(|seg| self.visit_path_segment_mut(seg))
    }
}
