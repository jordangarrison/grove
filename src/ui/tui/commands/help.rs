use super::*;

impl UiCommand {
    pub(super) fn help_hints_for(context: HelpHintContext) -> Vec<UiCommand> {
        Self::all()
            .iter()
            .filter_map(|command| command.help_hint_label(context).map(|_| *command))
            .collect()
    }

    pub(super) fn help_hint_label(self, context: HelpHintContext) -> Option<&'static str> {
        self.meta().help_hints.iter().find_map(|hint| {
            if hint.context == context {
                Some(hint.label)
            } else {
                None
            }
        })
    }
}
