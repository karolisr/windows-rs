use super::*;

/// Definition of a single command in a [`CommandBar`].
#[derive(Clone, Debug, PartialEq)]
pub enum CommandBarCommandDef {
    /// A clickable button with optional icon and label.
    Button {
        label: String,
        icon: Option<Icon>,
        tooltip: Option<String>,
    },
    /// A toggle button with optional icon and label.
    Toggle {
        label: String,
        icon: Option<Icon>,
        tooltip: Option<String>,
    },
    /// A visual separator.
    Separator,
}

impl CommandBarCommandDef {
    /// Sets the hover tooltip (`None` leaves WinUI's default, which is the
    /// label text once truncated/collapsed to an icon-only item).
    pub fn tooltip(mut self, text: impl Into<String>) -> Self {
        let tip = Some(text.into());
        match &mut self {
            Self::Button { tooltip, .. } | Self::Toggle { tooltip, .. } => {
                *tooltip = tip;
            }
            Self::Separator => {}
        }
        self
    }
}

/// Builder for a command bar button.
pub fn app_bar_button(label: impl Into<String>) -> CommandBarCommandDef {
    CommandBarCommandDef::Button {
        label: label.into(),
        icon: None,
        tooltip: None,
    }
}

/// Builder for a command bar button with icon.
pub fn app_bar_button_icon(label: impl Into<String>, icon: impl Into<Icon>) -> CommandBarCommandDef {
    CommandBarCommandDef::Button {
        label: label.into(),
        icon: Some(icon.into()),
        tooltip: None,
    }
}

/// Builder for a command bar toggle button.
pub fn app_bar_toggle(label: impl Into<String>) -> CommandBarCommandDef {
    CommandBarCommandDef::Toggle {
        label: label.into(),
        icon: None,
        tooltip: None,
    }
}

/// Builder for a command bar toggle button with icon.
pub fn app_bar_toggle_icon(label: impl Into<String>, icon: impl Into<Icon>) -> CommandBarCommandDef {
    CommandBarCommandDef::Toggle {
        label: label.into(),
        icon: Some(icon.into()),
        tooltip: None,
    }
}

/// Builder for a command bar separator.
pub fn app_bar_separator() -> CommandBarCommandDef {
    CommandBarCommandDef::Separator
}

/// `Microsoft.UI.Xaml.Controls.CommandBar`. A toolbar with primary and secondary commands.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct CommandBar {
    pub key: Option<String>,
    pub modifiers: Modifiers,
    pub primary_commands: Vec<CommandBarCommandDef>,
    pub secondary_commands: Vec<CommandBarCommandDef>,
    pub default_label_position: CommandBarDefaultLabelPosition,
    pub on_click: Option<Callback<String>>,
}

impl CommandBar {
    pub fn new(primary: Vec<CommandBarCommandDef>) -> Self {
        Self {
            primary_commands: primary,
            ..Default::default()
        }
    }

    pub fn secondary_commands(mut self, cmds: Vec<CommandBarCommandDef>) -> Self {
        self.secondary_commands = cmds;
        self
    }

    pub fn default_label_position(mut self, pos: CommandBarDefaultLabelPosition) -> Self {
        self.default_label_position = pos;
        self
    }

    pub fn on_click(mut self, f: impl IntoCallback<String>) -> Self {
        self.on_click = Some(f.into_callback());
        self
    }
}

impl Widget for CommandBar {
    widget_header!(ControlKind::CommandBar);
    fn bindings(&self) -> PropBindings {
        let mut out = generated::command_bar_bindings(self);
        out.push(Binding::Prop(
            Prop::PrimaryCommands,
            PropValue::CommandBarCommands(self.primary_commands.clone()),
        ));
        out.push(Binding::Prop(
            Prop::SecondaryCommands,
            PropValue::CommandBarCommands(self.secondary_commands.clone()),
        ));
        out
    }
}

pub fn command_bar(primary: Vec<CommandBarCommandDef>) -> CommandBar {
    CommandBar::new(primary)
}
