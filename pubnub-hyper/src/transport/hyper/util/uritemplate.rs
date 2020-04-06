//! This mod provides as easier to use interface for `uritemplate` crate.

use uritemplate::{IntoTemplateVar, TemplateVar};

pub struct UriTemplate(uritemplate::UriTemplate);

pub enum IfEmpty {
    #[allow(dead_code)]
    Set,
    #[allow(dead_code)]
    Dash,
    #[allow(dead_code)]
    Skip,
}

impl UriTemplate {
    #[allow(dead_code)]
    pub fn new(template: &str) -> Self {
        Self(uritemplate::UriTemplate::new(template))
    }

    #[allow(dead_code)]
    pub fn set_scalar(&mut self, varname: impl AsRef<str>, var: impl Into<String>) -> &mut Self {
        self.0
            .set(varname.as_ref(), TemplateVar::Scalar(var.into()));
        self
    }

    #[allow(dead_code)]
    pub fn set_optional_scalar(
        &mut self,
        varname: impl AsRef<str>,
        var: Option<impl Into<String>>,
    ) -> &mut Self {
        match var {
            Some(var) => self.set_scalar(varname, var),
            None => self,
        }
    }

    #[allow(dead_code)]
    pub fn set_list<T: Into<String>>(
        &mut self,
        varname: impl AsRef<str>,
        var: impl IntoIterator<Item = T>,
    ) -> &mut Self {
        self.0.set(
            varname.as_ref(),
            TemplateVar::List(var.into_iter().map(Into::into).collect()),
        );
        self
    }

    #[allow(dead_code)]
    pub fn set_list_with_if_empty<T: Into<String>>(
        &mut self,
        varname: impl AsRef<str>,
        var: impl IntoIterator<Item = T>,
        if_empty: IfEmpty,
    ) -> &mut Self {
        let mut var = var.into_iter();
        match var.next() {
            Some(first) => self.set_list(varname, std::iter::once(first).chain(var)),
            None => match if_empty {
                IfEmpty::Set => self.set_list(varname, std::iter::empty::<String>()),
                IfEmpty::Dash => self.set_scalar(varname, "-"),
                IfEmpty::Skip => self,
            },
        }
    }

    #[allow(dead_code)]
    pub fn set_assoc(
        &mut self,
        varname: impl AsRef<str>,
        var: impl IntoIterator<Item = (String, String)>,
    ) -> &mut Self {
        self.0.set(
            varname.as_ref(),
            TemplateVar::AssociativeArray(var.into_iter().collect()),
        );
        self
    }

    #[allow(dead_code)]
    pub fn set_template_var(
        &mut self,
        varname: impl AsRef<str>,
        var: impl IntoTemplateVar,
    ) -> &mut Self {
        self.0.set(varname.as_ref(), var);
        self
    }

    #[allow(dead_code)]
    pub fn tap(&mut self, f: impl FnOnce(&mut Self)) -> &mut Self {
        f(self);
        self
    }

    #[allow(dead_code)]
    pub fn delete(&mut self, varname: impl AsRef<str>) -> bool {
        self.0.delete(varname.as_ref())
    }

    #[allow(dead_code)]
    pub fn delete_all(&mut self) {
        self.0.delete_all()
    }

    #[allow(dead_code)]
    pub fn build(&mut self) -> String {
        self.0.build()
    }
}
