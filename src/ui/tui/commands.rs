#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum UiCommand {
    ToggleFocus,
    ToggleSidebar,
    OpenPreview,
    EnterInteractive,
    FocusPreview,
    FocusList,
    MoveSelectionUp,
    MoveSelectionDown,
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    ScrollBottom,
    PreviousTab,
    NextTab,
    NewWorkspace,
    EditWorkspace,
    StartAgent,
    StopAgent,
    DeleteWorkspace,
    MergeWorkspace,
    UpdateFromBase,
    OpenProjects,
    OpenSettings,
    ToggleUnsafe,
    OpenHelp,
    OpenCommandPalette,
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PaletteCommandSpec {
    pub(super) id: &'static str,
    pub(super) title: &'static str,
    pub(super) description: &'static str,
    pub(super) tags: &'static [&'static str],
    pub(super) category: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StatusHintContext {
    List,
    PreviewAgent,
    PreviewGit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum HelpHintContext {
    Global,
    Workspace,
    List,
    PreviewAgent,
    PreviewGit,
}

impl UiCommand {
    pub(super) const ALL: [UiCommand; 28] = [
        UiCommand::ToggleFocus,
        UiCommand::ToggleSidebar,
        UiCommand::OpenPreview,
        UiCommand::EnterInteractive,
        UiCommand::FocusPreview,
        UiCommand::FocusList,
        UiCommand::MoveSelectionUp,
        UiCommand::MoveSelectionDown,
        UiCommand::ScrollUp,
        UiCommand::ScrollDown,
        UiCommand::PageUp,
        UiCommand::PageDown,
        UiCommand::ScrollBottom,
        UiCommand::PreviousTab,
        UiCommand::NextTab,
        UiCommand::NewWorkspace,
        UiCommand::EditWorkspace,
        UiCommand::StartAgent,
        UiCommand::StopAgent,
        UiCommand::DeleteWorkspace,
        UiCommand::MergeWorkspace,
        UiCommand::UpdateFromBase,
        UiCommand::OpenProjects,
        UiCommand::OpenSettings,
        UiCommand::ToggleUnsafe,
        UiCommand::OpenHelp,
        UiCommand::OpenCommandPalette,
        UiCommand::Quit,
    ];

    pub(super) fn all() -> &'static [UiCommand] {
        &Self::ALL
    }

    pub(super) fn palette_spec(self) -> Option<PaletteCommandSpec> {
        match self {
            Self::ToggleFocus => Some(PaletteCommandSpec {
                id: "palette:toggle_focus",
                title: "Toggle Pane Focus",
                description: "Switch focus between workspace list and preview (Tab/h/l)",
                tags: &["tab", "h", "l", "focus", "pane"],
                category: "Navigation",
            }),
            Self::ToggleSidebar => Some(PaletteCommandSpec {
                id: "palette:toggle_sidebar",
                title: "Toggle Sidebar",
                description: "Show or hide workspace sidebar (\\)",
                tags: &["sidebar", "layout", "\\", "toggle"],
                category: "Navigation",
            }),
            Self::OpenPreview => Some(PaletteCommandSpec {
                id: "palette:open_preview",
                title: "Open Preview",
                description: "Focus preview pane for selected workspace (Enter/l)",
                tags: &["open", "preview", "enter", "l"],
                category: "List",
            }),
            Self::EnterInteractive => Some(PaletteCommandSpec {
                id: "palette:enter_interactive",
                title: "Enter Interactive Mode",
                description: "Attach to selected workspace session (Enter)",
                tags: &["interactive", "attach", "enter"],
                category: "Preview",
            }),
            Self::FocusPreview | Self::OpenCommandPalette => None,
            Self::FocusList => Some(PaletteCommandSpec {
                id: "palette:focus_list",
                title: "Focus Workspace List",
                description: "Return focus to workspace list (Esc)",
                tags: &["list", "focus", "esc"],
                category: "Navigation",
            }),
            Self::MoveSelectionUp => Some(PaletteCommandSpec {
                id: "palette:move_selection_up",
                title: "Select Previous Workspace",
                description: "Move workspace selection up (k / Up)",
                tags: &["up", "previous", "workspace", "k"],
                category: "List",
            }),
            Self::MoveSelectionDown => Some(PaletteCommandSpec {
                id: "palette:move_selection_down",
                title: "Select Next Workspace",
                description: "Move workspace selection down (j / Down)",
                tags: &["down", "next", "workspace", "j"],
                category: "List",
            }),
            Self::ScrollUp => Some(PaletteCommandSpec {
                id: "palette:scroll_up",
                title: "Scroll Up",
                description: "Scroll preview output up (k / Up)",
                tags: &["scroll", "up", "k"],
                category: "Preview",
            }),
            Self::ScrollDown => Some(PaletteCommandSpec {
                id: "palette:scroll_down",
                title: "Scroll Down",
                description: "Scroll preview output down (j / Down)",
                tags: &["scroll", "down", "j"],
                category: "Preview",
            }),
            Self::PageUp => Some(PaletteCommandSpec {
                id: "palette:page_up",
                title: "Page Up",
                description: "Scroll preview up by one page (PgUp)",
                tags: &["pageup", "pgup", "scroll"],
                category: "Preview",
            }),
            Self::PageDown => Some(PaletteCommandSpec {
                id: "palette:page_down",
                title: "Page Down",
                description: "Scroll preview down by one page (PgDn)",
                tags: &["pagedown", "pgdn", "scroll"],
                category: "Preview",
            }),
            Self::ScrollBottom => Some(PaletteCommandSpec {
                id: "palette:scroll_bottom",
                title: "Jump To Bottom",
                description: "Jump preview output to bottom (G)",
                tags: &["bottom", "latest", "G"],
                category: "Preview",
            }),
            Self::PreviousTab => Some(PaletteCommandSpec {
                id: "palette:previous_tab",
                title: "Previous Preview Tab",
                description: "Switch to previous preview tab ([)",
                tags: &["previous", "tab", "[", "agent", "git"],
                category: "Navigation",
            }),
            Self::NextTab => Some(PaletteCommandSpec {
                id: "palette:next_tab",
                title: "Next Preview Tab",
                description: "Switch to next preview tab (])",
                tags: &["next", "tab", "]", "agent", "git"],
                category: "Navigation",
            }),
            Self::NewWorkspace => Some(PaletteCommandSpec {
                id: "palette:new_workspace",
                title: "New Workspace",
                description: "Open workspace creation dialog (n)",
                tags: &["new", "workspace", "create", "n"],
                category: "Workspace",
            }),
            Self::EditWorkspace => Some(PaletteCommandSpec {
                id: "palette:edit_workspace",
                title: "Edit Workspace",
                description: "Open workspace edit dialog (e)",
                tags: &["edit", "workspace", "agent", "e"],
                category: "Workspace",
            }),
            Self::StartAgent => Some(PaletteCommandSpec {
                id: "palette:start_agent",
                title: "Start Agent",
                description: "Open start-agent dialog for selected workspace (s)",
                tags: &["start", "agent", "workspace", "s"],
                category: "Workspace",
            }),
            Self::StopAgent => Some(PaletteCommandSpec {
                id: "palette:stop_agent",
                title: "Stop Agent",
                description: "Stop selected workspace agent (x)",
                tags: &["stop", "agent", "workspace", "x"],
                category: "Workspace",
            }),
            Self::DeleteWorkspace => Some(PaletteCommandSpec {
                id: "palette:delete_workspace",
                title: "Delete Workspace",
                description: "Open delete dialog for selected workspace (D)",
                tags: &["delete", "workspace", "worktree", "D"],
                category: "Workspace",
            }),
            Self::MergeWorkspace => Some(PaletteCommandSpec {
                id: "palette:merge_workspace",
                title: "Merge Workspace",
                description: "Merge selected workspace branch into base (m)",
                tags: &["merge", "workspace", "branch", "m"],
                category: "Workspace",
            }),
            Self::UpdateFromBase => Some(PaletteCommandSpec {
                id: "palette:update_from_base",
                title: "Update From Base",
                description: "Merge base branch into selected workspace (u)",
                tags: &["update", "sync", "base", "workspace", "u"],
                category: "Workspace",
            }),
            Self::OpenProjects => Some(PaletteCommandSpec {
                id: "palette:open_projects",
                title: "Projects",
                description: "Open project switcher dialog (p)",
                tags: &["projects", "project", "switcher", "p"],
                category: "Workspace",
            }),
            Self::OpenSettings => Some(PaletteCommandSpec {
                id: "palette:open_settings",
                title: "Settings",
                description: "Open settings dialog (S)",
                tags: &["settings", "multiplexer", "S"],
                category: "Workspace",
            }),
            Self::ToggleUnsafe => Some(PaletteCommandSpec {
                id: "palette:toggle_unsafe",
                title: "Toggle Unsafe Launch",
                description: "Toggle launch skip-permissions default (!)",
                tags: &["unsafe", "permissions", "!"],
                category: "Workspace",
            }),
            Self::OpenHelp => Some(PaletteCommandSpec {
                id: "palette:open_help",
                title: "Keybind Help",
                description: "Open keyboard shortcut help (?)",
                tags: &["help", "shortcuts", "?"],
                category: "System",
            }),
            Self::Quit => Some(PaletteCommandSpec {
                id: "palette:quit",
                title: "Quit Grove",
                description: "Exit application (q)",
                tags: &["quit", "exit", "q"],
                category: "System",
            }),
        }
    }

    pub(super) fn from_palette_id(id: &str) -> Option<Self> {
        for command in Self::all() {
            if let Some(spec) = command.palette_spec()
                && spec.id == id
            {
                return Some(*command);
            }
        }
        None
    }

    pub(super) fn status_hints_for(context: StatusHintContext) -> &'static [UiCommand] {
        match context {
            StatusHintContext::List => &[
                UiCommand::MoveSelectionDown,
                UiCommand::FocusList,
                UiCommand::OpenPreview,
                UiCommand::NewWorkspace,
                UiCommand::EditWorkspace,
                UiCommand::MergeWorkspace,
                UiCommand::UpdateFromBase,
                UiCommand::OpenProjects,
                UiCommand::DeleteWorkspace,
                UiCommand::OpenSettings,
                UiCommand::ToggleSidebar,
                UiCommand::OpenCommandPalette,
                UiCommand::OpenHelp,
                UiCommand::Quit,
            ],
            StatusHintContext::PreviewAgent => &[
                UiCommand::PreviousTab,
                UiCommand::NextTab,
                UiCommand::ScrollDown,
                UiCommand::PageDown,
                UiCommand::ScrollBottom,
                UiCommand::FocusList,
                UiCommand::OpenPreview,
                UiCommand::NewWorkspace,
                UiCommand::EditWorkspace,
                UiCommand::MergeWorkspace,
                UiCommand::UpdateFromBase,
                UiCommand::OpenProjects,
                UiCommand::StartAgent,
                UiCommand::StopAgent,
                UiCommand::DeleteWorkspace,
                UiCommand::OpenSettings,
                UiCommand::ToggleSidebar,
                UiCommand::OpenCommandPalette,
                UiCommand::OpenHelp,
                UiCommand::Quit,
            ],
            StatusHintContext::PreviewGit => &[
                UiCommand::PreviousTab,
                UiCommand::NextTab,
                UiCommand::FocusList,
                UiCommand::EnterInteractive,
                UiCommand::NewWorkspace,
                UiCommand::EditWorkspace,
                UiCommand::MergeWorkspace,
                UiCommand::UpdateFromBase,
                UiCommand::OpenProjects,
                UiCommand::DeleteWorkspace,
                UiCommand::OpenSettings,
                UiCommand::ToggleSidebar,
                UiCommand::OpenCommandPalette,
                UiCommand::OpenHelp,
                UiCommand::Quit,
            ],
        }
    }

    pub(super) fn status_hint_label(self, context: StatusHintContext) -> Option<&'static str> {
        match (self, context) {
            (UiCommand::MoveSelectionDown, StatusHintContext::List) => Some("j/k move"),
            (UiCommand::FocusList, StatusHintContext::List)
            | (UiCommand::FocusList, StatusHintContext::PreviewAgent)
            | (UiCommand::FocusList, StatusHintContext::PreviewGit) => Some("h/l pane"),
            (UiCommand::OpenPreview, StatusHintContext::List)
            | (UiCommand::OpenPreview, StatusHintContext::PreviewAgent) => Some("Enter open"),
            (UiCommand::EnterInteractive, StatusHintContext::PreviewGit) => {
                Some("Enter attach lazygit")
            }
            (UiCommand::PreviousTab, StatusHintContext::PreviewAgent)
            | (UiCommand::PreviousTab, StatusHintContext::PreviewGit) => Some("[ prev tab"),
            (UiCommand::NextTab, StatusHintContext::PreviewAgent)
            | (UiCommand::NextTab, StatusHintContext::PreviewGit) => Some("] next tab"),
            (UiCommand::ScrollDown, StatusHintContext::PreviewAgent) => Some("j/k scroll"),
            (UiCommand::PageDown, StatusHintContext::PreviewAgent) => Some("PgUp/PgDn"),
            (UiCommand::ScrollBottom, StatusHintContext::PreviewAgent) => Some("G bottom"),
            (UiCommand::NewWorkspace, _context) => Some("n new"),
            (UiCommand::EditWorkspace, _context) => Some("e edit"),
            (UiCommand::MergeWorkspace, _context) => Some("m merge"),
            (UiCommand::UpdateFromBase, _context) => Some("u update"),
            (UiCommand::OpenProjects, _context) => Some("p projects"),
            (UiCommand::StartAgent, StatusHintContext::PreviewAgent) => Some("s start"),
            (UiCommand::StopAgent, StatusHintContext::PreviewAgent) => Some("x stop"),
            (UiCommand::DeleteWorkspace, _context) => Some("D delete"),
            (UiCommand::OpenSettings, _context) => Some("S settings"),
            (UiCommand::ToggleSidebar, _context) => Some("\\ sidebar"),
            (UiCommand::OpenCommandPalette, _context) => Some("Ctrl+K palette"),
            (UiCommand::OpenHelp, _context) => Some("? help"),
            (UiCommand::Quit, _context) => Some("q quit"),
            _ => None,
        }
    }

    pub(super) fn help_hints_for(context: HelpHintContext) -> &'static [UiCommand] {
        match context {
            HelpHintContext::Global => &[
                UiCommand::OpenHelp,
                UiCommand::Quit,
                UiCommand::ToggleFocus,
                UiCommand::ToggleSidebar,
                UiCommand::FocusList,
                UiCommand::OpenPreview,
                UiCommand::OpenCommandPalette,
            ],
            HelpHintContext::Workspace => &[
                UiCommand::NewWorkspace,
                UiCommand::EditWorkspace,
                UiCommand::MergeWorkspace,
                UiCommand::UpdateFromBase,
                UiCommand::OpenProjects,
                UiCommand::DeleteWorkspace,
                UiCommand::OpenSettings,
                UiCommand::ToggleUnsafe,
            ],
            HelpHintContext::List => &[UiCommand::MoveSelectionDown],
            HelpHintContext::PreviewAgent => &[
                UiCommand::PreviousTab,
                UiCommand::NextTab,
                UiCommand::ScrollDown,
                UiCommand::PageDown,
                UiCommand::ScrollBottom,
                UiCommand::StartAgent,
                UiCommand::StopAgent,
            ],
            HelpHintContext::PreviewGit => &[
                UiCommand::PreviousTab,
                UiCommand::NextTab,
                UiCommand::EnterInteractive,
            ],
        }
    }

    pub(super) fn help_hint_label(self, context: HelpHintContext) -> Option<&'static str> {
        match (self, context) {
            (UiCommand::OpenHelp, HelpHintContext::Global) => Some("? help"),
            (UiCommand::Quit, HelpHintContext::Global) => Some("q quit"),
            (UiCommand::ToggleFocus, HelpHintContext::Global) => Some("Tab/h/l switch pane"),
            (UiCommand::ToggleSidebar, HelpHintContext::Global) => Some("\\ toggle sidebar"),
            (UiCommand::FocusList, HelpHintContext::Global) => Some("Esc list pane"),
            (UiCommand::FocusPreview, HelpHintContext::Global) => Some("l preview pane"),
            (UiCommand::OpenPreview, HelpHintContext::Global) => Some("Enter open/attach"),
            (UiCommand::OpenCommandPalette, HelpHintContext::Global) => {
                Some("Ctrl+K command palette")
            }
            (UiCommand::NewWorkspace, HelpHintContext::Workspace) => Some("n new"),
            (UiCommand::EditWorkspace, HelpHintContext::Workspace) => Some("e edit"),
            (UiCommand::MergeWorkspace, HelpHintContext::Workspace) => Some("m merge"),
            (UiCommand::UpdateFromBase, HelpHintContext::Workspace) => Some("u update"),
            (UiCommand::OpenProjects, HelpHintContext::Workspace) => Some("p projects"),
            (UiCommand::DeleteWorkspace, HelpHintContext::Workspace) => Some("D delete"),
            (UiCommand::OpenSettings, HelpHintContext::Workspace) => Some("S settings"),
            (UiCommand::ToggleUnsafe, HelpHintContext::Workspace) => Some("! unsafe toggle"),
            (UiCommand::MoveSelectionDown, HelpHintContext::List) => {
                Some("j/k or Up/Down move selection")
            }
            (UiCommand::PreviousTab, HelpHintContext::PreviewAgent)
            | (UiCommand::PreviousTab, HelpHintContext::PreviewGit) => Some("[ prev tab"),
            (UiCommand::NextTab, HelpHintContext::PreviewAgent)
            | (UiCommand::NextTab, HelpHintContext::PreviewGit) => Some("] next tab"),
            (UiCommand::ScrollDown, HelpHintContext::PreviewAgent) => Some("j/k or Up/Down scroll"),
            (UiCommand::PageDown, HelpHintContext::PreviewAgent) => Some("PgUp/PgDn page"),
            (UiCommand::ScrollBottom, HelpHintContext::PreviewAgent) => Some("G bottom"),
            (UiCommand::StartAgent, HelpHintContext::PreviewAgent) => Some("s start"),
            (UiCommand::StopAgent, HelpHintContext::PreviewAgent) => Some("x stop"),
            (UiCommand::EnterInteractive, HelpHintContext::PreviewGit) => {
                Some("Enter attach lazygit")
            }
            _ => None,
        }
    }
}
