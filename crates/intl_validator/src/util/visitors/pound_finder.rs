use intl_markdown::{IcuPlural, IcuPound, Visit};

pub struct IcuPoundFinder {
    pound_instances: Vec<IcuPound>,
}

impl IcuPoundFinder {
    pub fn new() -> Self {
        Self {
            pound_instances: vec![],
        }
    }

    pub fn has_pound(&self) -> bool {
        self.pound_instances.len() > 0
    }

    pub fn instances(&self) -> &[IcuPound] {
        &self.pound_instances
    }
}

impl Visit for IcuPoundFinder {
    fn visit_icu_pound(&mut self, node: &IcuPound) {
        self.pound_instances.push(node.clone());
    }

    fn visit_icu_plural(&mut self, _node: &IcuPlural) {
        // Skip any nested plurals, since that resets the # context.
        return;
    }
}
